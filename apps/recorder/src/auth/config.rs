use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::{OneOrMany, serde_as};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicAuthConfig {
  #[serde(rename = "basic_user")]
  pub user: String,
  #[serde(rename = "basic_password")]
  pub password: String,
  #[serde(flatten)]
  pub session: AuthSessionConfig,
}

#[serde_as]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct OidcAuthConfig {
  #[serde(rename = "oidc_issuer")]
  pub issuer: String,
  #[serde(rename = "oidc_audience")]
  pub audience: String,
  #[serde(rename = "oidc_client_id")]
  pub client_id: String,
  #[serde(rename = "oidc_client_secret")]
  pub client_secret: String,
  #[serde(rename = "oidc_extra_scopes")]
  #[serde_as(as = "Option<OneOrMany<_>>")]
  pub extra_scopes: Option<Vec<String>>,
  #[serde(rename = "oidc_extra_claims")]
  pub extra_claims: Option<HashMap<String, Option<String>>>,
  #[serde(flatten)]
  pub session: AuthSessionConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "AuthWire", into = "AuthWire")]
pub enum AuthConfig {
  Basic(BasicAuthConfig),
  Oidc(OidcAuthConfig),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSessionConfig {
  pub identity_database_uri: String,
  pub external_base_url: url::Url,
  #[serde(default = "default_secure")]
  pub session_cookie_secure: bool,
  #[serde(default = "default_idle")]
  pub session_idle_seconds: u64,
  #[serde(default = "default_absolute")]
  pub session_absolute_seconds: i64,
  #[serde(default = "default_pending")]
  pub session_pending_seconds: i64,
  #[serde(default = "default_cleanup")]
  pub session_cleanup_seconds: u64,
}
fn default_secure() -> bool {
  true
}
fn default_idle() -> u64 {
  1800
}
fn default_absolute() -> i64 {
  43200
}
fn default_pending() -> i64 {
  300
}
fn default_cleanup() -> u64 {
  300
}

impl AuthConfig {
  pub fn session(&self) -> &AuthSessionConfig {
    match self {
      Self::Basic(config) => &config.session,
      Self::Oidc(config) => &config.session,
    }
  }
}

impl std::fmt::Debug for BasicAuthConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut debug = f.debug_struct("BasicAuthConfig");
    debug.field("user", &self.user);
    debug.field("session", &self.session);
    debug.field("password", &"[REDACTED]");
    debug.finish()
  }
}

impl std::fmt::Debug for OidcAuthConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut debug = f.debug_struct("OidcAuthConfig");
    debug.field("issuer", &self.issuer);
    debug.field("audience", &self.audience);
    debug.field("client_id", &self.client_id);
    debug.field("extra_scopes", &self.extra_scopes);
    debug.field("extra_claims", &self.extra_claims);
    debug.field("session", &self.session);
    debug.field("client_secret", &"[REDACTED]");
    debug.finish()
  }
}

impl std::fmt::Debug for AuthSessionConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut debug = f.debug_struct("AuthSessionConfig");
    debug.field("external_base_url", &self.external_base_url);
    debug.field("session_cookie_secure", &self.session_cookie_secure);
    debug.field("session_idle_seconds", &self.session_idle_seconds);
    debug.field("session_absolute_seconds", &self.session_absolute_seconds);
    debug.field("session_pending_seconds", &self.session_pending_seconds);
    debug.field("session_cleanup_seconds", &self.session_cleanup_seconds);
    debug.field("identity_database_uri", &"[REDACTED]");
    debug.finish()
  }
}

// The external schema separates login-provider credentials from session policy.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthWire {
  provider: ProviderWire,
  session: SessionWire,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum ProviderWire {
  Basic {
    username: String,
    password: String,
  },
  Oidc {
    issuer: String,
    audience: String,
    client_id: String,
    client_secret: String,
    extra_scopes: Option<Vec<String>>,
    extra_claims: Option<HashMap<String, Option<String>>>,
  },
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionWire {
  #[serde(default)]
  database_url: String,
  public_url: url::Url,
  #[serde(default = "default_secure")]
  cookie_secure: bool,
  #[serde(default = "default_idle")]
  idle_timeout_seconds: u64,
  #[serde(default = "default_absolute")]
  absolute_timeout_seconds: i64,
  #[serde(default = "default_pending")]
  login_timeout_seconds: i64,
  #[serde(default = "default_cleanup")]
  cleanup_interval_seconds: u64,
}
impl From<SessionWire> for AuthSessionConfig {
  fn from(w: SessionWire) -> Self {
    Self {
      identity_database_uri: w.database_url,
      external_base_url: w.public_url,
      session_cookie_secure: w.cookie_secure,
      session_idle_seconds: w.idle_timeout_seconds,
      session_absolute_seconds: w.absolute_timeout_seconds,
      session_pending_seconds: w.login_timeout_seconds,
      session_cleanup_seconds: w.cleanup_interval_seconds,
    }
  }
}
impl From<AuthSessionConfig> for SessionWire {
  fn from(c: AuthSessionConfig) -> Self {
    Self {
      database_url: c.identity_database_uri,
      public_url: c.external_base_url,
      cookie_secure: c.session_cookie_secure,
      idle_timeout_seconds: c.session_idle_seconds,
      absolute_timeout_seconds: c.session_absolute_seconds,
      login_timeout_seconds: c.session_pending_seconds,
      cleanup_interval_seconds: c.session_cleanup_seconds,
    }
  }
}
impl From<AuthWire> for AuthConfig {
  fn from(w: AuthWire) -> Self {
    let session = w.session.into();
    match w.provider {
      ProviderWire::Basic { username, password } => Self::Basic(BasicAuthConfig {
        user: username,
        password,
        session,
      }),
      ProviderWire::Oidc {
        issuer,
        audience,
        client_id,
        client_secret,
        extra_scopes,
        extra_claims,
      } => Self::Oidc(OidcAuthConfig {
        issuer,
        audience,
        client_id,
        client_secret,
        extra_scopes,
        extra_claims,
        session,
      }),
    }
  }
}
impl From<AuthConfig> for AuthWire {
  fn from(c: AuthConfig) -> Self {
    match c {
      AuthConfig::Basic(c) => Self {
        provider: ProviderWire::Basic {
          username: c.user,
          password: c.password,
        },
        session: c.session.into(),
      },
      AuthConfig::Oidc(c) => Self {
        provider: ProviderWire::Oidc {
          issuer: c.issuer,
          audience: c.audience,
          client_id: c.client_id,
          client_secret: c.client_secret,
          extra_scopes: c.extra_scopes,
          extra_claims: c.extra_claims,
        },
        session: c.session.into(),
      },
    }
  }
}
