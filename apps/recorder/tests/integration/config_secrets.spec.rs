//! Every environment scenario executes the production loader in an isolated
//! child.
use std::{
  path::PathBuf,
  process::{Command, Output},
};

use recorder::app::{AppConfig, Environment};
#[tokio::test]
async fn config_secrets_child() {
  if std::env::var("CONFIG_SECRETS_CHILD").is_err() {
    return;
  }
  AppConfig::load_dotenv(&Environment::Testing, Some(&std::env::var("CONFIG_SECRETS_DOTENV").unwrap()))
    .await
    .unwrap();
  let config = AppConfig::load_config(&Environment::Testing, Some(&std::env::var("CONFIG_SECRETS_PATH").unwrap())).await;
  if let Ok(category) = std::env::var("CONFIG_SECRETS_ERROR") {
    let error = config.unwrap_err();
    let text = format!("{error:?} {error}");
    assert!(text.contains(&category));
    println!("{text}");
  } else {
    let config = config.unwrap();
    assert!(!format!("{config:?}").contains("SENTINEL"), "AppConfig Debug leaked secret");
    let debug = format!("{:?} {:?} {:?}", config.auth, config.database, config.task);
    for secret in ["SENTINEL", "${FINAL}"] {
      assert!(!debug.contains(secret));
    }
    if let Ok(expected) = std::env::var("CONFIG_SECRETS_EXPECTED") {
      let expected: serde_json::Value = serde_json::from_str(&expected).unwrap();
      let actual = serde_json::to_value(config).unwrap();
      for (path, value) in expected.as_object().unwrap() {
        assert_eq!(actual.pointer(path), Some(value), "{path}");
      }
    }
  }
}
struct Fixture {
  root: PathBuf,
  path: PathBuf,
  dotenv: PathBuf,
}
impl Fixture {
  fn new() -> Self {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("../../temp/verification/fixtures/config")
      .join(uuid::Uuid::now_v7().to_string());
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("config.toml");
    let dotenv = root.join("empty.env");
    std::fs::write(&dotenv, "").unwrap();
    Self { root, path, dotenv }
  }
  fn config(&self, extra: &str) {
    std::fs::write(&self.path,format!("[server]\nport=5001\n[auth.provider]\ntype='basic'\nusername='fixture'\npassword='fixture'\n[auth.session]\ndatabase_url='postgres://fixture/identity'\npublic_url='http://127.0.0.1:5001'\n[database]\nurl='postgres://fixture/business'\n{extra}")).unwrap();
  }
  fn command(&self) -> Command {
    let mut c = Command::new(std::env::current_exe().unwrap());
    c.env_clear()
      .args(["--exact", "config_secrets_child", "--nocapture"])
      .env("CONFIG_SECRETS_CHILD", "1")
      .env("CONFIG_SECRETS_PATH", &self.path)
      .env("CONFIG_SECRETS_DOTENV", &self.dotenv);
    c
  }
  fn check(&self, c: &mut Command) {
    let output: Output = c.output().unwrap();
    let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success(), "{text}");
    assert!(!text.contains("SENTINEL"), "secret leaked");
  }
  fn secret(&self, name: &str, value: impl AsRef<[u8]>) -> PathBuf {
    let path = self.root.join(name);
    std::fs::write(&path, value).unwrap();
    path
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    std::fs::remove_dir_all(&self.root).unwrap();
  }
}
#[test]
fn config_secrets_interpolation_provenance_and_native_structure() {
  let f = Fixture::new();
  f.config(
    "[media]\nauto_optimize_formats=['$FORMAT','${UNSET:-image/webp}']\n[storage]\ndata_dir='''line 1\n${WORD}\n{{ quoted old template }}\n$${FINAL} $$ \
     $9'''\n",
  );
  let mut c = f.command();
  c.env("SERVER__HOST", "${FINAL}").env("FORMAT", "image/webp").env("WORD", "SENTINEL-WORD");
  // The sentinel only occurs in the explicitly asserted ordinary header, never
  // a diagnostic.
  c.env("WORD","second line").env("CONFIG_SECRETS_EXPECTED",r#"{"/server/host":"${FINAL}","/media/auto_optimize_formats":["image/webp","image/webp"],"/storage/data_dir":"line 1\nsecond line\n{{ quoted old template }}\n${FINAL} $ $9"}"#);
  f.check(&mut c);
  for expression in [
    "${MISSING}",
    "${EMPTY}",
    "${X:?message}",
    "${X:+a}",
    "${X-default}",
    "${X:-$Y}",
    "${X:-${Y}}",
    "$(command)",
    "${BROKEN",
  ] {
    f.config(&format!("[storage]\ndata_dir='{expression}'"));
    f.check(f.command().env("EMPTY", "").env(
      "CONFIG_SECRETS_ERROR",
      if expression == "${MISSING}" || expression == "${EMPTY}" {
        "missing"
      } else if expression == "${BROKEN" {
        "invalid"
      } else {
        "unsupported"
      },
    ));
  }
  f.config("[media]\njxl_effort=4\njxl_speed=3");
  f.check(f.command().env("CONFIG_SECRETS_ERROR", "media.jxl_effort"));
  f.config("[media]\njxl_distance=2.8\njxl_quality=80");
  f.check(f.command().env("CONFIG_SECRETS_ERROR", "media.jxl_distance"));
  f.config("[media]\njxl_quality=80");
  f.check(f.command().env("CONFIG_SECRETS_ERROR", "media.jxl_quality"));
  for extra in [
    "[scheduler.worker]\nconcurrency=2",
    "[database.pool]\nconnect_timeout=500",
    "[auth.session]\npublic_urll='http://fixture'",
    "[media.execution]\nconcurency=2",
  ] {
    f.config(extra);
    f.check(f.command().env("CONFIG_SECRETS_ERROR", "Invalid configuration"));
  }
  f.config("");
  f.check(
    f.command()
      .env("TASK__QUEUE_DATABASE_URI", "postgres://fixture/old")
      .env("CONFIG_SECRETS_ERROR", "removed"),
  );
  f.config("[storage]\ndata_dir='${EMPTY:-}'");
  f.check(f.command().env("EMPTY", "").env("CONFIG_SECRETS_EXPECTED", r#"{"/storage/data_dir":""}"#));
  let yaml = f.root.join("config.yaml");
  std::fs::write(&yaml,"server:\n  <<: &BASE\n    host: '${HOSTNAME:-fixture}'\n    port: 5001\n  port: 6008\nauth:\n  provider:\n    type: basic\n    username: fixture\n    password: fixture\n  session:\n    database_url: postgres://fixture/identity\n    public_url: http://127.0.0.1:5001\ndatabase:\n  url: postgres://fixture/business\nmedia:\n  auto_optimize_formats: [image/webp]\n").unwrap();
  f.check(
    f.command()
      .env("CONFIG_SECRETS_PATH", yaml)
      .env("CONFIG_SECRETS_EXPECTED", r#"{"/server/host":"fixture","/server/port":6008}"#),
  );
}
#[test]
fn config_secrets_six_maps_conflicts_and_literal_content() {
  let f = Fixture::new();
  f.config("");
  for (name, path, value) in [
    ("AUTH__PROVIDER__PASSWORD", "/auth/provider/password", "true"),
    (
      "AUTH__SESSION__DATABASE_URL",
      "/auth/session/database_url",
      "postgres://SENTINEL-id@fixture/identity",
    ),
    ("DATABASE__URL", "/database/url", "postgres://SENTINEL-db@fixture/business"),
    (
      "DATABASE__MIGRATION__URL",
      "/database/migration/url",
      "postgres://SENTINEL-owner@fixture/business",
    ),
    ("SCHEDULER__DATABASE__URL", "/scheduler/database/url", "postgres://SENTINEL-q@fixture/business"),
  ] {
    let file = f.secret(name, value);
    let expected = serde_json::json!({path:value}).to_string();
    f.check(f.command().env(format!("{name}_FILE"), &file).env("CONFIG_SECRETS_EXPECTED", expected));
    f.check(
      f.command()
        .env(format!("{name}_FILE"), &file)
        .env(name, "")
        .env("CONFIG_SECRETS_ERROR", "conflict"),
    );
    f.check(
      f.command()
        .env(name, value)
        .env(name.to_ascii_lowercase(), value)
        .env("CONFIG_SECRETS_ERROR", "duplicate"),
    );
  }
  let oidc = f.root.join("oidc.toml");
  std::fs::write(&oidc,"[server]\nport=5001\n[auth.provider]\ntype='oidc'\nissuer='http://127.0.0.1:5002/'\nclient_id='fixture'\naudience='fixture'\nclient_secret='fixture'\n[auth.session]\ndatabase_url='postgres://fixture/identity'\npublic_url='http://127.0.0.1:5001'\n[database]\nurl='postgres://fixture/business'").unwrap();
  let secret = "SENTINEL-'\"\n${FINAL}";
  let file = f.secret("oidc", secret);
  f.check(
    f.command()
      .env("CONFIG_SECRETS_PATH", oidc)
      .env("AUTH__PROVIDER__CLIENT_SECRET_FILE", file)
      .env(
        "CONFIG_SECRETS_EXPECTED",
        serde_json::json!({"/auth/provider/client_secret":secret}).to_string(),
      ),
  );
  let file = f.secret("password", secret);
  f.check(
    f.command()
      .env("AUTH__PROVIDER__PASSWORD_FILE", file)
      .env("CONFIG_SECRETS_EXPECTED", serde_json::json!({"/auth/provider/password":secret}).to_string()),
  );
  f.check(
    f.command()
      .env("DATABASE__PASSWORD_FILE", "/missing")
      .env("CONFIG_SECRETS_ERROR", "unsupported"),
  );
  f.check(f.command().env("SOME_TOOL_FILE", "/missing"));
}
#[test]
fn config_secrets_bounded_file_validation_and_symlink() {
  let f = Fixture::new();
  f.config("");
  let files = [
    f.secret("empty", b""),
    f.secret("big", vec![b'x'; 65537]),
    f.secret("utf8", [255]),
    f.root.clone(),
    f.root.join("missing"),
  ];
  for file in files {
    f.check(
      f.command()
        .env("AUTH__PROVIDER__PASSWORD_FILE", file)
        .env("CONFIG_SECRETS_ERROR", "file secret"),
    );
  }
  for value in ["", "relative"] {
    f.check(f.command().env("AUTH__PROVIDER__PASSWORD_FILE", value).env("CONFIG_SECRETS_ERROR", "absolute"));
  }
  let file = f.secret("readonly", b"123");
  let mut permissions = std::fs::metadata(&file).unwrap().permissions();
  permissions.set_readonly(true);
  std::fs::set_permissions(&file, permissions).unwrap();
  #[cfg(unix)]
  {
    let link = f.root.join("symlink");
    std::os::unix::fs::symlink(&file, &link).unwrap();
    f.check(
      f.command()
        .env("AUTH__PROVIDER__PASSWORD_FILE", link)
        .env("CONFIG_SECRETS_EXPECTED", r#"{"/auth/provider/password":"123"}"#),
    );
    let fifo = f.root.join("fifo");
    assert!(Command::new("mkfifo").arg(&fifo).status().unwrap().success());
    f.check(f.command().env("AUTH__PROVIDER__PASSWORD_FILE", fifo).env("CONFIG_SECRETS_ERROR", "regular"));
  }
  let newline = f.secret("uri", "postgres://SENTINEL@fixture/business\n");
  f.check(f.command().env("DATABASE__URL_FILE", newline).env("CONFIG_SECRETS_ERROR", "database.url"));
}
#[test]
fn config_secrets_actual_startup_errors_redact_all_sensitive_inputs() {
  let f = Fixture::new();
  f.config("[logger]\nenable=false\nlevel='off'\nformat='compact'");
  let output = Command::new(env!("CARGO_BIN_EXE_recorder-cli"))
    .env_clear()
    .args([
      "--environment",
      "testing",
      "--config-file",
      f.path.to_str().unwrap(),
      "--dotenv-file",
      f.dotenv.to_str().unwrap(),
    ])
    .env("AUTH__PROVIDER__PASSWORD", "SENTINEL-password-\"\n${FINAL}")
    .env("AUTH__SESSION__DATABASE_URL", "postgres://SENTINEL-identity@127.0.0.1:1/id")
    .env("DATABASE__URL", "postgres://SENTINEL-business@127.0.0.1:1/db")
    .env("DATABASE__MIGRATION__URL", "postgres://SENTINEL-migration@127.0.0.1:1/db")
    .env("SCHEDULER__DATABASE__URL", "postgres://SENTINEL-queue@127.0.0.1:1/db")
    .output()
    .unwrap();
  assert!(!output.status.success());
  let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
  assert!(!text.contains("SENTINEL"), "startup leaked secret");
  assert!(text.contains("redacted") || text.contains("unavailable"));
}
#[test]
fn config_secrets_backfill_dry_run_is_public_only_bounded_and_repeatable() {
  let f = Fixture::new();
  let data = f.root.join("data");
  std::fs::create_dir_all(data.join("public")).unwrap();
  std::fs::create_dir_all(data.join("subscribers/1001")).unwrap();
  for name in ["a.jpg", "b.png", "c.jpg"] {
    std::fs::write(data.join("public").join(name), b"dry-run-does-not-decode").unwrap();
  }
  std::fs::write(data.join("subscribers/1001/private.jpg"), b"private").unwrap();
  f.config(&format!("[storage]\ndata_dir='{}'", data.display()));
  let run = || {
    Command::new(env!("CARGO_BIN_EXE_media-backfill"))
      .env_clear()
      .args([
        "--config-file",
        f.path.to_str().unwrap(),
        "--dotenv-file",
        f.dotenv.to_str().unwrap(),
        "--limit",
        "1",
        "--offset",
        "1",
      ])
      .output()
      .unwrap()
  };
  let first = run();
  assert!(first.status.success());
  let output = String::from_utf8(first.stdout).unwrap();
  let plans = if cfg!(feature = "jxl") { 2 } else { 1 };
  assert!(output.contains(&format!("dry-run {plans}: public/b.png")));
  assert!(!output.contains("private"));
  assert!(!output.contains("a.jpg"));
  assert!(!output.contains("c.jpg"));
  assert_eq!(run().stdout, output.as_bytes());
  assert!(!data.join("public/b.png.derivatives.json").exists());
}
