mod client;
mod config;
mod ranges;
pub use client::{StorageContentCategory, StorageService, StorageStoredUrl};
pub use config::StorageConfig;

pub(crate) fn revalidation_body() -> axum::body::Body {
  // Axum's outer Route fills Content-Length from an exact size hint even for
  // 304. An empty stream prevents it inventing a zero representation length.
  axum::body::Body::from_stream(futures::stream::empty::<Result<bytes::Bytes, std::convert::Infallible>>())
}

pub fn merge_vary(headers: &mut http::HeaderMap, value: &str) {
  if headers
    .get_all(http::header::VARY)
    .iter()
    .filter_map(|v| v.to_str().ok())
    .flat_map(|v| v.split(','))
    .any(|v| v.trim().eq_ignore_ascii_case(value) || v.trim() == "*")
  {
    return;
  }
  headers.append(http::header::VARY, http::HeaderValue::from_str(value).expect("fixed Vary token"));
}
