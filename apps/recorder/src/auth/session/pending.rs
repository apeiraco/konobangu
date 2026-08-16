use securitydept_oidc_client::{OidcError, OidcResult, PendingOauth, PendingOauthStore, PendingOauthStoreConfig};
use serde::Deserialize;
use sqlx::{PgPool, Row};

#[derive(Clone, Deserialize)]
pub struct PendingStoreConfig {
  #[serde(skip)]
  pub pool: Option<PgPool>,
  #[serde(default = "default_ttl")]
  pub ttl_seconds: i64,
}

fn default_ttl() -> i64 {
  300
}
impl Default for PendingStoreConfig {
  fn default() -> Self {
    Self {
      pool: None,
      ttl_seconds: default_ttl(),
    }
  }
}
impl PendingOauthStoreConfig for PendingStoreConfig {}

pub struct PostgresPendingStore {
  pool: PgPool,
  ttl_seconds: i64,
}

fn store_error(source: sqlx::Error) -> OidcError {
  OidcError::PendingOauth { source: Box::new(source) }
}

impl PendingOauthStore for PostgresPendingStore {
  type Config = PendingStoreConfig;

  fn from_config(config: &Self::Config) -> Self {
    Self {
      pool: config.pool.clone().expect("Pending OAuth requires an explicitly configured identity pool"),
      ttl_seconds: config.ttl_seconds,
    }
  }

  async fn insert(&self, state: String, nonce: String, code_verifier: Option<String>, extra_data: Option<serde_json::Value>) -> OidcResult<()> {
    sqlx::query(
      "INSERT INTO auth_session.pending_oauth (state, nonce, code_verifier, extra_data, expires_at) VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP + $5 * INTERVAL \
       '1 second')",
    )
    .bind(state)
    .bind(nonce)
    .bind(code_verifier)
    .bind(extra_data)
    .bind(self.ttl_seconds)
    .execute(&self.pool)
    .await
    .map_err(store_error)?;
    Ok(())
  }

  async fn take(&self, state: &str) -> OidcResult<Option<PendingOauth>> {
    let row = sqlx::query(
      "DELETE FROM auth_session.pending_oauth WHERE state = $1 RETURNING nonce, code_verifier, extra_data, expires_at > CURRENT_TIMESTAMP AS valid",
    )
    .bind(state)
    .fetch_optional(&self.pool)
    .await
    .map_err(store_error)?;
    row
      .filter(|row| row.get::<bool, _>("valid"))
      .map(|row| {
        Ok(PendingOauth {
          nonce: row.try_get("nonce").map_err(store_error)?,
          code_verifier: row.try_get("code_verifier").map_err(store_error)?,
          extra_data: row.try_get("extra_data").map_err(store_error)?,
        })
      })
      .transpose()
  }
}
