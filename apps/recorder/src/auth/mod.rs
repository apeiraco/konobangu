pub mod basic;
pub mod config;
pub mod errors;
pub mod middleware;
pub mod oidc;
pub mod service;

pub use config::{AuthConfig, BasicAuthConfig, OidcAuthConfig};
pub use errors::AuthError;
pub use middleware::{auth_middleware, webui_auth_middleware};
pub use service::{AuthService, AuthServiceTrait, AuthUserInfo};
pub mod session;
