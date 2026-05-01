use std::{fmt::Debug, ops::Deref, sync::Arc, time::Duration};

use async_trait::async_trait;
use axum::http::Extensions;
use http_cache_reqwest::{
    Cache, CacheManager, CacheMode, HttpCache, HttpCacheOptions, MokaManager,
};
use leaky_bucket::RateLimiter;
use reqwest::{self, ClientBuilder, Request, Response};
use reqwest_cookie_store::{CookieStore, CookieStoreRwLock};
use reqwest_middleware::{
    ClientBuilder as ClientWithMiddlewareBuilder, ClientWithMiddleware, Middleware, Next,
};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_tracing::TracingMiddleware;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use util::OptDynErr;

use crate::{HttpClientError, client::proxy::HttpClientProxyConfig, get_random_ua};

pub struct RateLimiterMiddleware {
    rate_limiter: RateLimiter,
}

#[async_trait]
impl Middleware for RateLimiterMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &'_ mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        self.rate_limiter.acquire_one().await;
        next.run(req, extensions).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum HttpClientCacheBackendConfig {
    Moka { cache_size: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HttpClientCachePresetConfig {
    #[serde(rename = "rfc7234")]
    #[default]
    RFC7234,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HttpClientConfig {
    pub exponential_backoff_max_retries: Option<u32>,
    pub leaky_bucket_max_tokens: Option<u32>,
    pub leaky_bucket_initial_tokens: Option<u32>,
    pub leaky_bucket_refill_tokens: Option<u32>,
    #[serde_as(as = "Option<serde_with::DurationMilliSeconds>")]
    pub leaky_bucket_refill_interval: Option<Duration>,
    pub user_agent: Option<String>,
    pub cache_backend: Option<HttpClientCacheBackendConfig>,
    pub cache_preset: Option<HttpClientCachePresetConfig>,
    pub proxy: Option<HttpClientProxyConfig>,
}

pub(crate) enum CacheBackend {
    Moka(MokaManager),
}

impl CacheManager for CacheBackend {
    async fn get(
        &self,
        cache_key: &str,
    ) -> http_cache::Result<Option<(http_cache::HttpResponse, http_cache_semantics::CachePolicy)>>
    {
        match self {
            Self::Moka(manager) => manager.get(cache_key).await,
        }
    }

    /// Attempts to cache a response and related policy.
    async fn put(
        &self,
        cache_key: String,
        res: http_cache::HttpResponse,
        policy: http_cache_semantics::CachePolicy,
    ) -> http_cache::Result<http_cache::HttpResponse> {
        match self {
            Self::Moka(manager) => manager.put(cache_key, res, policy).await,
        }
    }

    /// Attempts to remove a record from cache.
    async fn delete(&self, cache_key: &str) -> http_cache::Result<()> {
        match self {
            Self::Moka(manager) => manager.delete(cache_key).await,
        }
    }
}

pub trait HttpClientTrait: Deref<Target = ClientWithMiddleware> + Debug {}

pub struct HttpClientFork {
    pub client_builder: ClientBuilder,
    pub middleware_stack: Vec<Arc<dyn Middleware>>,
    pub config: HttpClientConfig,
    pub cookie_store: Option<Arc<CookieStoreRwLock>>,
}

impl HttpClientFork {
    pub fn attach_cookies(mut self, cookies: Option<&str>) -> Result<Self, HttpClientError> {
        let cookie_store = if let Some(cookies) = cookies {
            serde_json::from_str(cookies)
                .map_err(|err| HttpClientError::ParseCookiesError { source: err })?
        } else {
            CookieStore::default()
        };

        let cookies_store = Arc::new(CookieStoreRwLock::new(cookie_store));

        self.cookie_store = Some(cookies_store.clone());
        self.client_builder = self.client_builder.cookie_provider(cookies_store);
        Ok(self)
    }

    pub fn attach_user_agent(mut self, user_agent: &str) -> Self {
        self.client_builder = self.client_builder.user_agent(user_agent);
        self
    }
}

pub struct HttpClient {
    pub cookie_store: Option<Arc<CookieStoreRwLock>>,
    client: ClientWithMiddleware,
    middleware_stack: Vec<Arc<dyn Middleware>>,
    pub config: HttpClientConfig,
}

impl Debug for HttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClient")
            .field("config", &self.config)
            .finish()
    }
}

impl From<HttpClient> for ClientWithMiddleware {
    fn from(val: HttpClient) -> Self {
        val.client
    }
}

impl Deref for HttpClient {
    type Target = ClientWithMiddleware;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl HttpClient {
    pub fn from_config(config: HttpClientConfig) -> Result<Self, HttpClientError> {
        let mut middleware_stack: Vec<Arc<dyn Middleware>> = vec![];
        let mut reqwest_client_builder = ClientBuilder::new().user_agent(
            config
                .user_agent
                .as_deref()
                .unwrap_or_else(|| get_random_ua()),
        );

        if let Some(proxy) = config.proxy.as_ref() {
            let accept_invalid_certs = proxy
                .accept_invalid_certs
                .as_ref()
                .map(|b| *b)
                .unwrap_or_default();
            let proxy = proxy.clone().into_proxy()?;
            if let Some(proxy) = proxy {
                reqwest_client_builder = reqwest_client_builder.proxy(proxy);
                if accept_invalid_certs {
                    reqwest_client_builder =
                        reqwest_client_builder.danger_accept_invalid_certs(true);
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        let reqwest_client_builder =
            reqwest_client_builder.redirect(reqwest::redirect::Policy::none());

        let reqwest_client = reqwest_client_builder.build()?;
        let mut reqwest_with_middleware_builder = ClientWithMiddlewareBuilder::new(reqwest_client);

        {
            let tracing_middleware = Arc::new(TracingMiddleware::default());

            middleware_stack.push(tracing_middleware.clone());

            reqwest_with_middleware_builder =
                reqwest_with_middleware_builder.with_arc(tracing_middleware)
        }

        {
            if let Some(ref x) = config.exponential_backoff_max_retries {
                let retry_policy = ExponentialBackoff::builder().build_with_max_retries(*x);

                let retry_transient_middleware =
                    Arc::new(RetryTransientMiddleware::new_with_policy(retry_policy));

                middleware_stack.push(retry_transient_middleware.clone());

                reqwest_with_middleware_builder =
                    reqwest_with_middleware_builder.with_arc(retry_transient_middleware);
            }
        }

        {
            if let (None, None, None, None) = (
                config.leaky_bucket_initial_tokens.as_ref(),
                config.leaky_bucket_refill_tokens.as_ref(),
                config.leaky_bucket_refill_interval.as_ref(),
                config.leaky_bucket_max_tokens.as_ref(),
            ) {
            } else {
                let mut rate_limiter_builder = RateLimiter::builder();

                if let Some(ref x) = config.leaky_bucket_max_tokens {
                    rate_limiter_builder.max(*x as usize);
                }
                if let Some(ref x) = config.leaky_bucket_initial_tokens {
                    rate_limiter_builder.initial(*x as usize);
                }
                if let Some(ref x) = config.leaky_bucket_refill_tokens {
                    rate_limiter_builder.refill(*x as usize);
                }
                if let Some(ref x) = config.leaky_bucket_refill_interval {
                    rate_limiter_builder.interval(*x);
                }

                let rate_limiter = rate_limiter_builder.build();

                let rate_limiter_middleware = Arc::new(RateLimiterMiddleware { rate_limiter });

                middleware_stack.push(rate_limiter_middleware.clone());

                reqwest_with_middleware_builder =
                    reqwest_with_middleware_builder.with_arc(rate_limiter_middleware);
            }
        }

        {
            if let (None, None) = (config.cache_backend.as_ref(), config.cache_preset.as_ref()) {
            } else {
                let cache_preset = config.cache_preset.as_ref().cloned().unwrap_or_default();
                let cache_backend = config
                    .cache_backend
                    .as_ref()
                    .map(|b| match b {
                        HttpClientCacheBackendConfig::Moka { cache_size } => {
                            CacheBackend::Moka(MokaManager {
                                cache: Arc::new(moka::future::Cache::new(*cache_size)),
                            })
                        }
                    })
                    .unwrap_or_else(|| CacheBackend::Moka(MokaManager::default()));

                let http_cache = match cache_preset {
                    HttpClientCachePresetConfig::RFC7234 => HttpCache {
                        mode: CacheMode::Default,
                        manager: cache_backend,
                        options: HttpCacheOptions::default(),
                    },
                };

                let http_cache_middleware = Arc::new(Cache(http_cache));

                middleware_stack.push(http_cache_middleware.clone());

                reqwest_with_middleware_builder =
                    reqwest_with_middleware_builder.with_arc(http_cache_middleware);
            }
        }

        let reqwest_with_middleware = reqwest_with_middleware_builder.build();

        Ok(Self {
            client: reqwest_with_middleware,
            middleware_stack,
            config,
            cookie_store: None,
        })
    }

    pub fn fork(&self) -> HttpClientFork {
        let mut reqwest_client_builder = ClientBuilder::new().user_agent(
            self.config
                .user_agent
                .as_deref()
                .unwrap_or_else(|| get_random_ua()),
        );

        if let Some(proxy) = self.config.proxy.as_ref() {
            let accept_invalid_certs = proxy
                .accept_invalid_certs
                .as_ref()
                .map(|b| *b)
                .unwrap_or_default();
            let proxy = proxy.clone().into_proxy().unwrap_or_default();
            if let Some(proxy) = proxy {
                reqwest_client_builder = reqwest_client_builder.proxy(proxy);
                if accept_invalid_certs {
                    reqwest_client_builder =
                        reqwest_client_builder.danger_accept_invalid_certs(true);
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        let reqwest_client_builder =
            reqwest_client_builder.redirect(reqwest::redirect::Policy::none());

        HttpClientFork {
            client_builder: reqwest_client_builder,
            middleware_stack: self.middleware_stack.clone(),
            config: self.config.clone(),
            cookie_store: self.cookie_store.clone(),
        }
    }

    pub fn from_fork(fork: HttpClientFork) -> Result<Self, HttpClientError> {
        let HttpClientFork {
            client_builder,
            middleware_stack,
            config,
            cookie_store,
        } = fork;
        let reqwest_client = client_builder.build()?;
        let mut reqwest_with_middleware_builder = ClientWithMiddlewareBuilder::new(reqwest_client);

        for m in &middleware_stack {
            reqwest_with_middleware_builder = reqwest_with_middleware_builder.with_arc(m.clone());
        }

        let reqwest_with_middleware = reqwest_with_middleware_builder.build();

        Ok(Self {
            client: reqwest_with_middleware,
            middleware_stack,
            config,
            cookie_store,
        })
    }

    pub fn save_cookie_store_to_json(&self) -> Result<Option<String>, HttpClientError> {
        if let Some(cookie_store) = self.cookie_store.as_ref() {
            let json = {
                let cookie_store =
                    cookie_store
                        .read()
                        .map_err(|_| HttpClientError::SaveCookiesError {
                            message: "Failed to read cookie store".to_string(),
                            source: None.into(),
                        })?;

                if cookie_store.iter_any().next().is_none() {
                    return Ok(None);
                }

                serde_json::to_string(&cookie_store as &CookieStore).map_err(|err| {
                    HttpClientError::SaveCookiesError {
                        message: "Failed to serialize cookie store".to_string(),
                        source: OptDynErr::some_boxed(err),
                    }
                })?
            };

            Ok(Some(json))
        } else {
            Ok(None)
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        HttpClient::from_config(Default::default()).expect("Failed to create default HttpClient")
    }
}

impl HttpClientTrait for HttpClient {}
