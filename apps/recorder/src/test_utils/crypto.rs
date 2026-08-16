use crate::{
  crypto::{CryptoConfig, CryptoService},
  errors::RecorderResult,
};

pub async fn build_testing_crypto_service() -> RecorderResult<CryptoService> {
  let crypto = CryptoService::from_config(CryptoConfig {}).await?;
  Ok(crypto)
}
