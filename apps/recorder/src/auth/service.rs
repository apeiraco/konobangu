use async_trait::async_trait;
use axum::http::request::Parts;
use http::header::HeaderValue;

use super::{AuthConfig, basic::BasicAuthService, errors::AuthError, oidc::OidcAuthService};
use crate::{app::AppContextTrait, models::auth::AuthType};

#[derive(Clone, Debug)]
pub struct AuthUserInfo {
  pub subscriber_auth: crate::models::auth::Model,
  pub auth_type: AuthType,
}

#[async_trait]
pub trait AuthServiceTrait {
  async fn extract_user_info(&self, ctx: &dyn AppContextTrait, request: &mut Parts) -> Result<AuthUserInfo, AuthError>;
  fn www_authenticate_header_value(&self) -> Option<HeaderValue>;
  fn auth_type(&self) -> AuthType;
}

pub enum AuthService {
  Basic(Box<BasicAuthService>),
  Oidc(Box<OidcAuthService>),
}

impl AuthService {
  pub async fn from_conf(config: AuthConfig) -> Result<Self, AuthError> {
    let runtime = super::session::runtime::SessionRuntime::new(config.session().clone()).await?;
    match config {
      AuthConfig::Basic(config) => Ok(Self::Basic(Box::new(BasicAuthService { config, runtime }))),
      AuthConfig::Oidc(config) => Ok(Self::Oidc(Box::new(OidcAuthService::new(config, runtime).await?))),
    }
  }

  pub fn runtime(&self) -> &super::session::runtime::SessionRuntime {
    match self {
      Self::Basic(service) => &service.runtime,
      Self::Oidc(service) => &service.runtime,
    }
  }

  pub fn identity_db(&self) -> &sea_orm::DatabaseConnection {
    &self.runtime().identity_db
  }
}

#[async_trait]
impl AuthServiceTrait for AuthService {
  #[tracing::instrument(skip(self, ctx, request))]
  async fn extract_user_info(&self, ctx: &dyn AppContextTrait, request: &mut Parts) -> Result<AuthUserInfo, AuthError> {
    match self {
      AuthService::Basic(service) => service.extract_user_info(ctx, request).await,
      AuthService::Oidc(service) => service.extract_user_info(ctx, request).await,
    }
  }

  fn www_authenticate_header_value(&self) -> Option<HeaderValue> {
    match self {
      AuthService::Basic(service) => service.www_authenticate_header_value(),
      AuthService::Oidc(service) => service.www_authenticate_header_value(),
    }
  }

  fn auth_type(&self) -> AuthType {
    match self {
      AuthService::Basic(service) => service.auth_type(),
      AuthService::Oidc(service) => service.auth_type(),
    }
  }
}
