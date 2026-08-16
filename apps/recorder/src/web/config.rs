use serde::{Deserialize, Serialize};

use super::middleware::MiddlewareConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebServerConfig {
  /// The address on which the server should listen on for incoming
  /// connections.
  #[serde(default = "default_binding")]
  pub binding: String,
  /// The port on which the server should listen for incoming connections.
  #[serde(default = "default_port")]
  pub port: u16,
  /// The webserver host
  pub host: String,
  /// Identify via the `Server` header
  pub ident: Option<String>,
  /// Middleware configurations for the server, including payload limits,
  /// logging, and error handling.
  #[serde(default)]
  pub middlewares: MiddlewareConfig,
}

pub fn default_binding() -> String {
  "127.0.0.1".to_string()
}

pub fn default_port() -> u16 {
  5_001
}
