//! Reuse library parsing and range matching, then apply server preference ties.
use headers_accept::Accept;
use mediatype::{MediaType, ReadParams, WriteParams, names};

use crate::errors::RecorderResult;
pub fn validate_accept(accept: &Accept) -> RecorderResult<()> {
  for range in accept.media_types() {
    if range.params().filter(|(name, _)| *name == names::Q).count() > 1 {
      return Err(super::invalid_options("Invalid Accept header"));
    }
    if let Some(q) = range.get_param(names::Q) {
      let s = q.as_str();
      let (whole, fraction) = s.split_once('.').unwrap_or((s, ""));
      let valid = matches!(whole, "0" | "1") && fraction.len() <= 3 && fraction.bytes().all(|c| c.is_ascii_digit() && (whole == "0" || c == b'0'));
      if !valid {
        return Err(super::invalid_options("Invalid Accept header"));
      }
    }
  }
  Ok(())
}
pub fn quality(accept: &Accept, mime: &str, explicit: bool) -> Option<f32> {
  let candidate = MediaType::parse(mime).ok()?;
  // The iterator is already sorted by specificity by headers-accept.
  for range in accept.media_types() {
    if explicit && range.essence() != candidate.essence() {
      continue;
    }
    let mut matcher = range.to_ref();
    matcher.remove_params(names::Q);
    let one = Accept::from_iter([matcher]);
    if one.negotiate([&candidate]).is_some() {
      return Some(range.get_param(names::Q).map_or(1.0, |q| q.as_str().parse().unwrap_or(0.0)));
    }
  }
  None
}
pub fn choose(accept: &Accept, candidates: &[(&str, bool)]) -> Option<usize> {
  let mut best = None;
  let mut best_q = 0.0;
  for (index, (mime, explicit)) in candidates.iter().enumerate() {
    if let Some(q) = quality(accept, mime, *explicit)
      && q > best_q
    {
      best_q = q;
      best = Some(index);
    }
  }
  best
}
