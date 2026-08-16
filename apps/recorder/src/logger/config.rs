use serde::{Deserialize, Serialize};

use super::{
  LogRotation,
  core::{LogFormat, LogLevel},
};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct LoggerConfig {
  pub enable: bool,

  #[serde(default)]
  pub pretty_backtrace: bool,

  pub level: LogLevel,

  pub format: LogFormat,

  pub override_filter: Option<String>,

  pub file_appender: Option<LoggerFileAppender>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct LoggerFileAppender {
  pub enable: bool,
  #[serde(default)]
  pub non_blocking: bool,
  pub level: LogLevel,
  pub format: LogFormat,
  pub rotation: LogRotation,
  pub dir: Option<String>,
  pub filename_prefix: Option<String>,
  pub filename_suffix: Option<String>,
  pub max_log_files: usize,
}
