use async_trait::async_trait;
use axum::http::{HeaderValue, request::Parts};
use base64::{self, Engine};
use http::header::AUTHORIZATION;

use super::{
  config::BasicAuthConfig,
  errors::AuthError,
  service::{AuthServiceTrait, AuthUserInfo},
};
use crate::{
  app::{AppContextTrait, PROJECT_NAME},
  models::{auth::AuthType, subscribers::SEED_SUBSCRIBER},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AuthBasic {
  pub user: String,
  pub password: Option<String>,
}

impl AuthBasic {
  fn decode_request_parts(req: &mut Parts) -> Result<Self, AuthError> {
    let authorization = req
      .headers
      .get(AUTHORIZATION)
      .and_then(|s| s.to_str().ok())
      .ok_or(AuthError::BasicInvalidCredentials)?;

    let split = authorization.split_once(' ');

    match split {
      Some(("Basic", contents)) => {
        let decoded = base64::engine::general_purpose::STANDARD
          .decode(contents)
          .map_err(|_| AuthError::BasicInvalidCredentials)?;

        let decoded = String::from_utf8(decoded).map_err(|_| AuthError::BasicInvalidCredentials)?;

        Ok(if let Some((user, password)) = decoded.split_once(':') {
          Self {
            user: String::from(user),
            password: Some(String::from(password)),
          }
        } else {
          Self { user: decoded, password: None }
        })
      }
      _ => Err(AuthError::BasicInvalidCredentials),
    }
  }
}

pub struct BasicAuthService {
  pub config: BasicAuthConfig,
  pub runtime: super::session::runtime::SessionRuntime,
}

#[async_trait]
impl AuthServiceTrait for BasicAuthService {
  async fn extract_user_info(&self, ctx: &dyn AppContextTrait, request: &mut Parts) -> Result<AuthUserInfo, AuthError> {
    if let Ok(AuthBasic {
      user: found_user,
      password: found_password,
    }) = AuthBasic::decode_request_parts(request)
      && self.config.user == found_user
      && self.config.password == found_password.unwrap_or_default()
    {
      let subscriber_auth = crate::models::auth::Model::find_by_pid(ctx, SEED_SUBSCRIBER)
        .await
        .map_err(|_| AuthError::FindAuthRecordError)?;
      return Ok(AuthUserInfo {
        subscriber_auth,
        auth_type: AuthType::Basic,
      });
    }
    Err(AuthError::BasicInvalidCredentials)
  }

  fn www_authenticate_header_value(&self) -> Option<HeaderValue> {
    Some(HeaderValue::from_str(format!("Basic realm=\"{PROJECT_NAME}\"").as_str()).unwrap())
  }

  fn auth_type(&self) -> AuthType {
    AuthType::Basic
  }
}
