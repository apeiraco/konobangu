use axum::{
  extract::FromRequestParts,
  http::{HeaderName, HeaderValue, Uri, header, request::Parts},
};
use itertools::Itertools;
use url::Url;

use crate::errors::RecorderError;

/// Fields from a "Forwarded" header per [RFC7239 sec 4](https://www.rfc-editor.org/rfc/rfc7239#section-4)
#[derive(Debug, Clone)]
pub struct ForwardedHeader {
  pub for_field: Vec<String>,
  pub by: Option<String>,
  pub host: Option<String>,
  pub proto: Option<String>,
}

impl ForwardedHeader {
  /// Return the 'for' headers as a list of [std::net::IpAddr]'s.
  pub fn for_as_ipaddr(self) -> Vec<std::net::IpAddr> {
    self
      .for_field
      .iter()
      .filter_map(|ip| {
        if ip.contains(']') {
          // this is an IPv6 address, get what's between the []
          ip.split(']').next()?.split('[').next_back()?.parse::<std::net::IpAddr>().ok()
        } else {
          ip.parse::<std::net::IpAddr>().ok()
        }
      })
      .collect::<Vec<std::net::IpAddr>>()
  }
}

/// This parses the Forwarded header, and returns a list of the IPs in the
/// "for=" fields. Per [RFC7239 sec 4](https://www.rfc-editor.org/rfc/rfc7239#section-4)
impl TryFrom<HeaderValue> for ForwardedHeader {
  type Error = String;
  fn try_from(forwarded: HeaderValue) -> Result<ForwardedHeader, String> {
    ForwardedHeader::try_from(&forwarded)
  }
}

/// This parses the Forwarded header, and returns a list of the IPs in the
/// "for=" fields. Per [RFC7239 sec 4](https://www.rfc-editor.org/rfc/rfc7239#section-4)
impl TryFrom<&HeaderValue> for ForwardedHeader {
  type Error = String;
  fn try_from(forwarded: &HeaderValue) -> Result<ForwardedHeader, String> {
    let mut for_field: Vec<String> = Vec::new();
    let mut by: Option<String> = None;
    let mut host: Option<String> = None;
    let mut proto: Option<String> = None;
    // first get the k=v pairs
    forwarded.to_str().map_err(|err| err.to_string())?.split(';').for_each(|s| {
      let s = s.trim().to_lowercase();
      // The for value can look like this:
      // for=192.0.2.43, for=198.51.100.17
      // so we need to handle this case
      if s.starts_with("for=") || s.starts_with("for =") {
        // we have a valid thing to grab
        let chunks: Vec<String> = s
          .split(',')
          .filter_map(|chunk| chunk.trim().split('=').next_back().map(|c| c.to_string()))
          .collect::<Vec<String>>();
        for_field.extend(chunks);
      } else if s.starts_with("by=") {
        by = s.split('=').next_back().map(|c| c.to_string());
      } else if s.starts_with("host=") {
        host = s.split('=').next_back().map(|c| c.to_string());
      } else if s.starts_with("proto=") {
        proto = s.split('=').next_back().map(|c| c.to_string());
      } else {
        // probably need to work out what to do here
      }
    });

    Ok(ForwardedHeader { for_field, by, host, proto })
  }
}

#[derive(Clone, Debug)]
pub struct ForwardedRelatedInfo {
  pub forwarded: Option<ForwardedHeader>,
  pub x_forwarded_proto: Option<String>,
  pub x_forwarded_host: Option<String>,
  pub x_forwarded_for: Option<Vec<String>>,
  pub host: Option<String>,
  pub uri: Uri,
  pub origin: Option<String>,
}

impl<T> FromRequestParts<T> for ForwardedRelatedInfo {
  type Rejection = RecorderError;
  fn from_request_parts(parts: &mut Parts, _state: &T) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
    let headers = &parts.headers;
    let forwarded = headers.get(header::FORWARDED).and_then(|s| ForwardedHeader::try_from(s.clone()).ok());

    let x_forwarded_proto = headers
      .get(HeaderName::from_static("x-forwarded-proto"))
      .and_then(|s| s.to_str().map(String::from).ok());

    let x_forwarded_host = headers
      .get(HeaderName::from_static("x-forwarded-host"))
      .and_then(|s| s.to_str().map(String::from).ok());

    let x_forwarded_for = headers
      .get(HeaderName::from_static("x-forwarded-for"))
      .and_then(|s| s.to_str().ok())
      .and_then(|s| {
        let l = s.split(",").map(|s| s.trim().to_string()).collect_vec();
        if l.is_empty() { None } else { Some(l) }
      });

    let host = headers.get(header::HOST).and_then(|s| s.to_str().map(String::from).ok());

    let origin = headers.get(header::ORIGIN).and_then(|s| s.to_str().map(String::from).ok());

    futures::future::ready(Ok(ForwardedRelatedInfo {
      host,
      x_forwarded_for,
      x_forwarded_host,
      x_forwarded_proto,
      forwarded,
      uri: parts.uri.clone(),
      origin,
    }))
  }
}

impl ForwardedRelatedInfo {
  pub fn resolved_protocol(&self) -> Option<&str> {
    self
      .forwarded
      .as_ref()
      .and_then(|s| s.proto.as_deref())
      .or(self.x_forwarded_proto.as_deref())
      .or(self.uri.scheme_str())
  }

  pub fn resolved_host(&self) -> Option<&str> {
    self
      .forwarded
      .as_ref()
      .and_then(|s| s.host.as_deref())
      .or(self.x_forwarded_host.as_deref())
      .or(self.host.as_deref())
      .or(self.uri.host())
  }

  pub fn resolved_origin(&self) -> Option<Url> {
    if let (Some(protocol), Some(host)) = (self.resolved_protocol(), self.resolved_host()) {
      let origin = format!("{protocol}://{host}");
      Url::parse(&origin).ok()
    } else {
      None
    }
  }
}
