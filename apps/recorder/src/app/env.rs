use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
#[value(rename_all = "snake_case")]
pub enum Environment {
  #[serde(alias = "dev")]
  #[value(alias = "dev")]
  Development,
  #[serde(alias = "prod")]
  #[value(alias = "prod")]
  Production,
  #[serde(alias = "test")]
  #[value(alias = "test")]
  Testing,
}

impl Environment {
  pub fn full_name(&self) -> &'static str {
    match &self {
      Self::Development => "development",
      Self::Production => "production",
      Self::Testing => "testing",
    }
  }

  pub fn short_name(&self) -> &'static str {
    match &self {
      Self::Development => "dev",
      Self::Production => "prod",
      Self::Testing => "test",
    }
  }
}
