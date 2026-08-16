use super::CacheConfig;
use crate::errors::RecorderResult;

pub struct CacheService {}

impl CacheService {
  pub async fn from_config(_config: CacheConfig) -> RecorderResult<Self> {
    Ok(Self {})
  }
}
