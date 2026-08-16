use std::ops::RangeInclusive;

use http::{HeaderMap, header};
use http_range::{HttpRange, HttpRangeParseError};

pub(super) enum Ranges {
  Ignore,
  Invalid,
  Bytes(Vec<RangeInclusive<u64>>),
}

pub(super) fn normalize(headers: &HeaderMap, len: u64) -> Ranges {
  if len == 0 || !headers.contains_key(header::RANGE) {
    return Ranges::Ignore;
  }
  let mut values = headers.get_all(header::RANGE).iter();
  let Some(value) = values.next().and_then(|v| v.to_str().ok()) else {
    return Ranges::Invalid;
  };
  if values.next().is_some() {
    return Ranges::Invalid;
  }
  if !value.starts_with("bytes=") {
    return Ranges::Ignore;
  }
  // Bound the parser's list allocation, including unsatisfiable members.
  if value.bytes().filter(|&byte| byte == b',').take(16).count() == 16 {
    return Ranges::Ignore;
  }
  let parsed = match HttpRange::parse(value, len) {
    Ok(parsed) => parsed,
    Err(HttpRangeParseError::NoOverlap) => return Ranges::Bytes(Vec::new()),
    Err(HttpRangeParseError::InvalidRange) => return Ranges::Invalid,
  };
  let mut ranges = Vec::new();
  for range in parsed {
    let Some(end) = range.start.checked_add(range.length) else {
      return Ranges::Invalid;
    };
    if range.length == 0 || range.start >= len || end > len {
      return Ranges::Invalid;
    }
    ranges.push(range.start..=end - 1);
  }
  // Overlap and excessive multipart counts are ignored rather than amplified.
  for (index, range) in ranges.iter().enumerate() {
    if ranges[index + 1..]
      .iter()
      .any(|other| range.start() <= other.end() && other.start() <= range.end())
    {
      return Ranges::Ignore;
    }
  }
  Ranges::Bytes(ranges)
}
