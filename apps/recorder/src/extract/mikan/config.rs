use fetch::HttpClientConfig;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MikanConfig {
  pub http_client: HttpClientConfig,
  pub base_url: Url,
}
