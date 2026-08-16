use super::MessageConfig;
use crate::errors::RecorderResult;

pub struct MessageService {
  pub config: MessageConfig,
}

impl MessageService {
  pub async fn from_config(config: MessageConfig) -> RecorderResult<Self> {
    Ok(Self { config })
  }
}
