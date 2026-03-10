use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use axum::http::request::Parts;
use fetch::{
    HttpClient, HttpClientConfig,
    client::{HttpClientCacheBackendConfig, HttpClientCachePresetConfig},
};
use http::header::HeaderValue;
use jwtk::jwk::RemoteJwksVerifier;
use moka::future::Cache;
use openidconnect::{IssuerUrl, core::CoreProviderMetadata};
use snafu::prelude::*;

use super::{
    AuthConfig,
    basic::BasicAuthService,
    errors::{AuthError, OidcProviderHttpClientSnafu, OidcProviderUrlSnafu},
    oidc::{OidcAuthService, OidcHttpClient},
};
use crate::{app::AppContextTrait, models::auth::AuthType};

#[derive(Clone, Debug)]
pub struct AuthUserInfo {
    pub subscriber_auth: crate::models::auth::Model,
    pub auth_type: AuthType,
}

#[async_trait]
pub trait AuthServiceTrait {
    async fn extract_user_info(
        &self,
        ctx: &dyn AppContextTrait,
        request: &mut Parts,
    ) -> Result<AuthUserInfo, AuthError>;
    fn www_authenticate_header_value(&self) -> Option<HeaderValue>;
    fn auth_type(&self) -> AuthType;
}

pub enum AuthService {
    Basic(Box<BasicAuthService>),
    Oidc(Box<OidcAuthService>),
}

impl AuthService {
    pub async fn from_conf(config: AuthConfig) -> Result<Self, AuthError> {
        let result = match config {
            AuthConfig::Basic(config) => AuthService::Basic(Box::new(BasicAuthService { config })),
            AuthConfig::Oidc(config) => {
                let oidc_provider_client = Arc::new(
                    HttpClient::from_config(HttpClientConfig {
                        exponential_backoff_max_retries: Some(3),
                        cache_backend: Some(HttpClientCacheBackendConfig::Moka { cache_size: 1 }),
                        cache_preset: Some(HttpClientCachePresetConfig::RFC7234),
                        ..Default::default()
                    })
                    .context(OidcProviderHttpClientSnafu)?,
                );

                let provider_metadata = {
                    let client = OidcHttpClient(oidc_provider_client.clone());
                    let issuer_url =
                        IssuerUrl::new(config.issuer.clone()).context(OidcProviderUrlSnafu)?;
                    CoreProviderMetadata::discover_async(issuer_url, &client).await
                }?;

                let jwk_verifier = RemoteJwksVerifier::builder(
                    provider_metadata.jwks_uri().to_string().clone(),
                )
                .with_cache_duration(Duration::from_secs(300))
                .build();

                AuthService::Oidc(Box::new(OidcAuthService {
                    config,
                    jwk_verifier,
                    oidc_provider_client,
                    oidc_request_cache: Cache::builder()
                        .time_to_live(Duration::from_mins(5))
                        .name("oidc_request_cache")
                        .build(),
                }))
            }
        };
        Ok(result)
    }
}

#[async_trait]
impl AuthServiceTrait for AuthService {
    #[tracing::instrument(skip(self, ctx, request))]
    async fn extract_user_info(
        &self,
        ctx: &dyn AppContextTrait,
        request: &mut Parts,
    ) -> Result<AuthUserInfo, AuthError> {
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
