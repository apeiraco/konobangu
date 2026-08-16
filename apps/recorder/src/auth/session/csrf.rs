use http::{HeaderMap, StatusCode};
use url::Url;

/// Cookie writes require all three checks; SameSite and CORS are not
/// authorization.
pub fn validate_cookie_write(headers: &HeaderMap, external_base_url: &Url) -> Result<(), StatusCode> {
  let expected_origin = external_base_url.origin().ascii_serialization();
  let origin = headers.get(http::header::ORIGIN).and_then(|value| value.to_str().ok());
  let csrf = headers.get("x-konobangu-csrf").and_then(|value| value.to_str().ok());
  let content_type = headers
    .get(http::header::CONTENT_TYPE)
    .and_then(|value| value.to_str().ok())
    .and_then(|value| value.split(';').next())
    .map(str::trim);
  if origin != Some(expected_origin.as_str()) || csrf != Some("1") {
    return Err(StatusCode::FORBIDDEN);
  }
  if !content_type.is_some_and(|value| value.eq_ignore_ascii_case("application/json")) {
    return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
  }
  Ok(())
}

/// Validate the untrusted return path before starting or completing
/// authentication.
pub fn validate_return_path(path: &str, origin: &Url) -> Result<String, StatusCode> {
  let mut decoded = path.to_owned();
  for _ in 0..4 {
    if !decoded.starts_with('/') || decoded.starts_with("//") || decoded.contains('\\') || decoded.chars().any(char::is_control) {
      return Err(StatusCode::BAD_REQUEST);
    }
    let next = percent_encoding::percent_decode_str(&decoded)
      .decode_utf8()
      .map_err(|_| StatusCode::BAD_REQUEST)?
      .into_owned();
    if next == decoded {
      break;
    }
    decoded = next;
  }
  if decoded.contains('%') || !decoded.starts_with('/') || decoded.starts_with("//") || decoded.contains('\\') || decoded.chars().any(char::is_control) {
    return Err(StatusCode::BAD_REQUEST);
  }
  let target = origin.join(&decoded).map_err(|_| StatusCode::BAD_REQUEST)?;
  if target.origin() != origin.origin() {
    return Err(StatusCode::BAD_REQUEST);
  }
  Ok(path.to_owned())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cookie_writes_require_origin_header_and_json() {
    let base = Url::parse("https://app.example/").unwrap();
    let mut headers = HeaderMap::new();
    assert_eq!(validate_cookie_write(&headers, &base), Err(StatusCode::FORBIDDEN));
    headers.insert("origin", "https://app.example".parse().unwrap());
    headers.insert("x-konobangu-csrf", "1".parse().unwrap());
    headers.insert("content-type", "application/json; charset=utf-8".parse().unwrap());
    assert_eq!(validate_cookie_write(&headers, &base), Ok(()));
    headers.insert("origin", "https://attacker.example".parse().unwrap());
    assert_eq!(validate_cookie_write(&headers, &base), Err(StatusCode::FORBIDDEN));
    headers.insert("origin", "https://app.example".parse().unwrap());
    headers.insert("content-type", "text/plain".parse().unwrap());
    assert_eq!(validate_cookie_write(&headers, &base), Err(StatusCode::UNSUPPORTED_MEDIA_TYPE));
  }

  #[test]
  fn return_paths_reject_encoded_external_targets() {
    let base = Url::parse("https://app.example/").unwrap();
    for path in [
      "https://evil.example",
      "//evil.example",
      "/%2fevil.example",
      "/%252fevil.example",
      "/\\evil.example",
      "/%5cevil.example",
      "/%0aevil",
    ] {
      assert!(validate_return_path(path, &base).is_err(), "{path}");
    }
    assert_eq!(validate_return_path("/subscriptions?tab=active", &base).unwrap(), "/subscriptions?tab=active");
  }
}
