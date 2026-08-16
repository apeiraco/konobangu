//! Resolve only effective file leaves; environment and secret bytes stay final.
use std::{
  collections::{BTreeMap, BTreeSet},
  fs::File,
  io::Read,
  path::Path,
  sync::OnceLock,
};

use figment::{
  Figment, Metadata, Profile, Provider,
  value::{Dict, Map, Tag, Value},
};

use crate::errors::{RecorderError, RecorderResult};

pub type Snapshot = BTreeMap<String, String>;
pub static ENVIRONMENT: OnceLock<Snapshot> = OnceLock::new();
pub const SECRETS: &[&str] = &[
  "AUTH__PROVIDER__PASSWORD",
  "AUTH__PROVIDER__CLIENT_SECRET",
  "AUTH__SESSION__DATABASE_URL",
  "DATABASE__URL",
  "DATABASE__MIGRATION__URL",
  "SCHEDULER__DATABASE__URL",
];
const NAMESPACES: &[&str] = &[
  "SERVER",
  "CACHE",
  "AUTH",
  "STORAGE",
  "MIKAN",
  "CRYPTO",
  "GRAPHQL",
  "MEDIA",
  "LOGGER",
  "DATABASE",
  "SCHEDULER",
  "MESSAGE",
];

pub fn invalid(path: &str, category: &str) -> RecorderError {
  RecorderError::InvalidConfiguration {
    message: format!("Invalid configuration at {path}: {category} (values redacted; legacy templates are not evaluated)"),
  }
}
#[derive(Clone)]
pub struct Values(pub Map<Profile, Dict>);
impl Provider for Values {
  fn metadata(&self) -> Metadata {
    Metadata::named("Resolved configuration")
  }
  fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
    Ok(self.0.clone())
  }
}

pub fn tags(fig: &Figment) -> RecorderResult<BTreeSet<Tag>> {
  fn visit(value: &Value, tags: &mut BTreeSet<Tag>) {
    tags.insert(value.tag());
    match value {
      Value::Dict(_, d) => {
        for v in d.values() {
          visit(v, tags);
        }
      }
      Value::Array(_, a) => {
        for v in a {
          visit(v, tags);
        }
      }
      _ => (),
    }
  }
  let mut tags = BTreeSet::new();
  for dict in fig.data().map_err(|_| invalid("file", "native parse failed"))?.values() {
    for v in dict.values() {
      visit(v, &mut tags);
    }
  }
  Ok(tags)
}
pub fn canonical(snapshot: &Snapshot) -> RecorderResult<Values> {
  let mut normalized = BTreeMap::new();
  for (key, value) in snapshot {
    let upper = key.to_ascii_uppercase();
    if upper.starts_with("TASK__") || matches!(upper.as_str(), "HOST" | "DATABASE_URL") {
      return Err(invalid(
        &upper.to_ascii_lowercase(),
        "removed configuration alias; use the documented service path",
      ));
    }
    if !upper.contains("__") || !NAMESPACES.contains(&upper.split("__").next().unwrap_or("")) {
      continue;
    }
    let path = upper.to_ascii_lowercase().replace("__", ".");
    if normalized.insert(upper, value).is_some() {
      return Err(invalid(&path, "duplicate canonical environment target"));
    }
  }
  let mut dict = Dict::new();
  for (name, value) in &normalized {
    let (target, value) = if let Some(target) = name.strip_suffix("_FILE") {
      if !SECRETS.contains(&target) {
        return Err(invalid(&name.to_ascii_lowercase().replace("__", "."), "unsupported file secret"));
      }
      if normalized.contains_key(target) {
        return Err(invalid(&target.to_ascii_lowercase().replace("__", "."), "value/file conflict"));
      }
      (target, Value::from(read_secret(value, &target.to_ascii_lowercase().replace("__", "."))?))
    } else {
      (
        name.as_str(),
        if SECRETS.contains(&name.as_str()) {
          Value::from(value.as_str())
        } else {
          value.parse().expect("Figment Value parsing is infallible")
        },
      )
    };
    insert(&mut dict, &target.to_ascii_lowercase().replace("__", "."), value)?;
  }
  Ok(Values(Map::from([(Profile::Default, dict)])))
}
fn insert(dict: &mut Dict, path: &str, value: Value) -> RecorderResult<()> {
  if let Some((head, rest)) = path.split_once('.') {
    let entry = dict.entry(head.into()).or_insert_with(|| Value::Dict(Tag::default(), Dict::new()));
    if let Value::Dict(_, child) = entry {
      insert(child, rest, value)
    } else {
      Err(invalid(path, "conflicting environment structure"))
    }
  } else {
    dict.insert(path.into(), value);
    Ok(())
  }
}
fn read_secret(path: &str, field: &str) -> RecorderResult<String> {
  if path.is_empty() || !Path::new(path).is_absolute() {
    return Err(invalid(field, "file secret path must be absolute and nonempty"));
  }
  // Check metadata before open so named pipes cannot block startup.
  let metadata = std::fs::metadata(path).map_err(|_| invalid(field, "unreadable file secret"))?;
  if !metadata.is_file() || metadata.len() > 65536 {
    return Err(invalid(field, "file secret must be a regular file of at most 64 KiB"));
  }
  let file = File::open(path).map_err(|_| invalid(field, "unreadable file secret"))?;
  if !file.metadata().map_err(|_| invalid(field, "unreadable file secret"))?.is_file() {
    return Err(invalid(field, "invalid file secret"));
  }
  let mut bytes = Vec::new();
  file.take(65537).read_to_end(&mut bytes).map_err(|_| invalid(field, "unreadable file secret"))?;
  if bytes.is_empty() || bytes.len() > 65536 {
    return Err(invalid(field, "empty or oversized file secret"));
  }
  String::from_utf8(bytes).map_err(|_| invalid(field, "file secret must be UTF-8"))
}

pub fn interpolate(fig: &Figment, file_tags: &BTreeSet<Tag>, snapshot: &Snapshot) -> RecorderResult<Values> {
  fn visit(value: &mut Value, path: &str, tags: &BTreeSet<Tag>, env: &Snapshot) -> RecorderResult<()> {
    match value {
      Value::String(tag, s) if tags.contains(tag) => *s = expand(s, path, env)?,
      Value::Dict(_, dict) => {
        for (key, value) in dict {
          visit(value, &format!("{path}.{key}"), tags, env)?;
        }
      }
      Value::Array(_, array) => {
        for (i, value) in array.iter_mut().enumerate() {
          visit(value, &format!("{path}[{i}]"), tags, env)?;
        }
      }
      _ => (),
    }
    Ok(())
  }
  let mut data = fig.data().map_err(|_| invalid("file", "native parse failed"))?;
  for dict in data.values_mut() {
    for (key, value) in dict {
      visit(value, key, file_tags, snapshot)?;
    }
  }
  Ok(Values(data))
}
fn name_start(c: u8) -> bool {
  c.is_ascii_alphabetic() || c == b'_'
}
fn name_char(c: u8) -> bool {
  name_start(c) || c.is_ascii_digit()
}
fn valid_name(s: &str) -> bool {
  s.as_bytes().first().is_some_and(|c| name_start(*c)) && s.bytes().all(name_char)
}
fn append(result: &mut String, text: &str, field: &str) -> RecorderResult<()> {
  if result.len().checked_add(text.len()).is_none_or(|len| len > 1024 * 1024) {
    return Err(invalid(field, "interpolation output limit"));
  }
  result.push_str(text);
  Ok(())
}

pub fn expand(input: &str, field: &str, env: &Snapshot) -> RecorderResult<String> {
  if input.len() > 1024 * 1024 {
    return Err(invalid(field, "interpolation input limit"));
  }
  let bytes = input.as_bytes();
  let mut i = 0;
  let mut result = String::new();
  while i < bytes.len() {
    if bytes[i] != b'$' {
      let start = i;
      i += 1;
      while i < bytes.len() && bytes[i] != b'$' {
        i += 1;
      }
      append(&mut result, &input[start..i], field)?;
      continue;
    }
    if bytes.get(i + 1) == Some(&b'$') {
      append(&mut result, "$", field)?;
      i += 2;
      continue;
    }
    let start = i;
    let (name, default);
    if bytes.get(i + 1) == Some(&b'{') {
      let end = input[i + 2..]
        .find('}')
        .map(|j| i + 2 + j)
        .ok_or_else(|| invalid(field, "invalid interpolation"))?;
      let inner = &input[i + 2..end];
      let parts = inner.split_once(":-");
      name = parts.map_or(inner, |p| p.0);
      default = parts.map(|p| p.1);
      if !valid_name(name) || default.is_some_and(|d| d.contains(['$', '{', '}'])) {
        return Err(invalid(field, "unsupported interpolation"));
      }
      i = end + 1;
    } else if bytes.get(i + 1).is_some_and(|c| name_start(*c)) {
      i += 2;
      while i < bytes.len() && name_char(bytes[i]) {
        i += 1;
      }
      name = &input[start + 1..i];
      default = None;
    } else {
      if bytes.get(i + 1) == Some(&b'(') {
        return Err(invalid(field, "unsupported interpolation"));
      }
      append(&mut result, "$", field)?;
      i += 1;
      continue;
    }
    let present = env.get(name).filter(|v| !v.is_empty());
    if present.is_none() && default.is_none() {
      return Err(invalid(field, "missing interpolation variable"));
    }
    // Invoke the library only on a validated expression; output is never
    // scanned again.
    let expanded = shellexpand::env_with_context(&input[start..i], |_| -> Result<Option<&str>, ()> {
      present.map(|s| Some(s.as_str())).ok_or(())
    })
    .map_err(|_| invalid(field, "missing interpolation variable"))?;
    append(&mut result, &expanded, field)?;
  }
  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn interpolation_bounds_include_literal_suffix_and_final_values() {
    let env = Snapshot::from([("WORD".into(), "x".repeat(1024 * 1024 - 4))]);
    assert!(
      expand("$WORD suffix", "storage.data_dir", &env)
        .unwrap_err()
        .to_string()
        .contains("output limit")
    );
    assert!(expand(&"x".repeat(1024 * 1024 + 1), "storage.data_dir", &env).is_err());
    let env = Snapshot::from([("WORD".into(), "${UNSET}".into())]);
    assert_eq!(expand("$WORD", "storage.data_dir", &env).unwrap(), "${UNSET}");
  }
}
