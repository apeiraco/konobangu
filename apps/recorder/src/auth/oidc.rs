use async_trait::async_trait;
use axum::http::{HeaderValue, request::Parts};
use securitydept_oidc_client::{OidcClient, OidcClientConfig};
use tower_sessions::Session;

use super::{
  AuthError,
  config::OidcAuthConfig,
  service::{AuthServiceTrait, AuthUserInfo},
  session::{
    pending::{PendingStoreConfig, PostgresPendingStore},
    runtime::SessionRuntime,
  },
};
use crate::{app::AppContextTrait, models::auth::AuthType};

pub struct OidcAuthService {
  pub client: OidcClient<PostgresPendingStore>,
  pub runtime: SessionRuntime,
}

impl OidcAuthService {
  pub async fn new(config: OidcAuthConfig, runtime: SessionRuntime) -> Result<Self, AuthError> {
    let initialized = async {
      // Access-token audience/custom-claim checks cannot silently become
      // ID-token checks.
      if config.audience != config.client_id || config.extra_claims.as_ref().is_some_and(|claims| !claims.is_empty()) {
        return Err(AuthError::SessionConfiguration {
          message: "Legacy oidc_audience must equal oidc_client_id and oidc_extra_claims must be empty; move non-equivalent access-token restrictions to the \
                    IdP before migration"
            .into(),
        });
      }
      let scopes = config.extra_scopes.as_ref().map(|value| value.to_vec()).unwrap_or_default();
      let mut requested = vec!["openid".to_owned(), "profile".to_owned()];
      requested.extend(scopes.clone());
      let mut sdk: OidcClientConfig<PendingStoreConfig> = serde_json::from_value(serde_json::json!({
          "client_id": config.client_id, "client_secret": config.client_secret,
          "issuer_url": config.issuer,
          "well_known_url": format!("{}/.well-known/openid-configuration", config.issuer.trim_end_matches('/')),
          "scopes": requested, "required_scopes": scopes, "pkce_enabled": true,
          "redirect_url": "/api/auth/session/callback"
      }))
      .map_err(|_| AuthError::SessionConfiguration {
        message: "OIDC configuration is invalid".into(),
      })?;
      sdk.pending_store = Some(PendingStoreConfig {
        pool: Some(runtime.identity_db.get_postgres_connection_pool().clone()),
        ttl_seconds: runtime.config.session_pending_seconds,
      });
      let client = OidcClient::from_config(sdk).await.map_err(|_| AuthError::SessionConfiguration {
        message: "OIDC discovery/configuration failed; no fallback authentication is enabled".into(),
      })?;
      Ok(client)
    }
    .await;
    match initialized {
      Ok(client) => Ok(Self { client, runtime }),
      Err(error) => {
        let _ = runtime.close().await;
        Err(error)
      }
    }
  }
}

#[async_trait]
impl AuthServiceTrait for OidcAuthService {
  async fn extract_user_info(&self, _ctx: &dyn AppContextTrait, request: &mut Parts) -> Result<AuthUserInfo, AuthError> {
    let session = request.extensions.get::<Session>().ok_or(AuthError::SessionUnauthorized)?;
    let subscriber_auth = self.runtime.authenticate(session).await?;
    if subscriber_auth.auth_type != AuthType::Oidc {
      return Err(AuthError::SessionUnauthorized);
    }
    Ok(AuthUserInfo {
      subscriber_auth,
      auth_type: AuthType::Oidc,
    })
  }
  fn www_authenticate_header_value(&self) -> Option<HeaderValue> {
    None
  }
  fn auth_type(&self) -> AuthType {
    AuthType::Oidc
  }
}
