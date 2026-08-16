use serde::{Deserialize, Serialize};
use serde_variant::to_variant_name;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub enum LogLevel {
  #[serde(rename = "off")]
  Off,
  #[serde(rename = "trace")]
  Trace,
  #[serde(rename = "debug")]
  Debug,
  #[serde(rename = "info")]
  #[default]
  Info,
  #[serde(rename = "warn")]
  Warn,
  #[serde(rename = "error")]
  Error,
}

impl std::fmt::Display for LogLevel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    to_variant_name(self).expect("only enum supported").fmt(f)
  }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub enum LogFormat {
  #[serde(rename = "compact")]
  #[default]
  Compact,
  #[serde(rename = "pretty")]
  Pretty,
  #[serde(rename = "json")]
  Json,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub enum LogRotation {
  #[serde(rename = "minutely")]
  Minutely,
  #[serde(rename = "hourly")]
  #[default]
  Hourly,
  #[serde(rename = "daily")]
  Daily,
  #[serde(rename = "never")]
  Never,
}
