use std::ops::Bound;

use http::HeaderValue;

pub fn build_no_satisfiable_content_range(len: u64) -> HeaderValue {
  HeaderValue::from_str(&format!("bytes */{len}")).unwrap_or_else(|e| unreachable!("Invalid content range: {e}"))
}

pub fn bound_range_to_content_range(r: &(Bound<u64>, Bound<u64>), l: u64) -> Option<HeaderValue> {
  match r {
    (Bound::Included(start), Bound::Included(end)) => Some(format!("bytes {start}-{end}/{l}")),
    (Bound::Included(start), Bound::Excluded(end)) => Some(format!("bytes {start}-{}/{l}", end - 1)),
    (Bound::Included(start), Bound::Unbounded) => Some(format!("bytes {start}-{}/{l}", if l > 0 { l - 1 } else { 0 })),
    _ => None,
  }
  .and_then(|s| HeaderValue::from_str(&s).ok())
}
