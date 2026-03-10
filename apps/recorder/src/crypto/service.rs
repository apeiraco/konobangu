use base64::prelude::{BASE64_URL_SAFE, *};
use cocoon::Cocoon;
use rand::RngExt;
use serde::{Deserialize, Serialize};

use super::CryptoConfig;
use crate::crypto::error::CryptoError;

pub struct CryptoService {
    #[allow(dead_code)]
    config: CryptoConfig,
}

impl CryptoService {
    pub async fn from_config(config: CryptoConfig) -> Result<Self, CryptoError> {
        Ok(Self { config })
    }

    pub fn encrypt_string(&self, data: String) -> Result<String, CryptoError> {
        let key = rand::rng().random::<[u8; 32]>();
        let mut cocoon = Cocoon::new(&key);

        let mut data = data.into_bytes();

        let detached_prefix = cocoon.encrypt(&mut data)?;

        let mut combined = Vec::with_capacity(key.len() + detached_prefix.len() + data.len());
        combined.extend_from_slice(&key);
        combined.extend_from_slice(&detached_prefix);
        combined.extend_from_slice(&data);

        Ok(BASE64_URL_SAFE.encode(combined))
    }

    pub fn decrypt_string(&self, data: &str) -> Result<String, CryptoError> {
        let decoded = BASE64_URL_SAFE.decode(data)?;

        let (key, remain) = decoded.split_at(32);
        let (detached_prefix, data) = remain.split_at(60);
        let mut data = data.to_vec();
        let cocoon = Cocoon::new(key);

        cocoon.decrypt(&mut data, detached_prefix)?;

        String::from_utf8(data).map_err(CryptoError::from)
    }

    pub fn encrypt_serialize<T: Serialize>(&self, credentials: &T) -> Result<String, CryptoError> {
        let json = serde_json::to_string(credentials)?;

        self.encrypt_string(json)
    }

    pub fn decrypt_deserialize<T: for<'de> Deserialize<'de>>(
        &self,
        encrypted: &str,
    ) -> Result<T, CryptoError> {
        let data = self.decrypt_string(encrypted)?;

        serde_json::from_str(&data).map_err(CryptoError::from)
    }
}
