use async_graphql::dynamic::ResolverContext;
use axum::{
  Json,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use snafu::prelude::*;

use crate::models::auth::AuthType;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum AuthError {
  #[snafu(display("Authentication configuration is invalid: {message}"))]
  SessionConfiguration { message: String },
  #[snafu(display("Authentication storage is unavailable"))]
  SessionStorage,
  #[snafu(display("Authentication is required"))]
  SessionUnauthorized,

  #[snafu(display("Permission denied"))]
  PermissionError,
  #[snafu(display("Not support auth method"))]
  NotSupportAuthMethod { supported: Vec<AuthType>, current: AuthType },
  #[snafu(display("Failed to find auth record"))]
  FindAuthRecordError,
  #[snafu(display("Invalid credentials"))]
  BasicInvalidCredentials,
  #[snafu(display(
        "GraphQL permission denied since {context_path}{}{field}{}{column}: {}",
        (if field.is_empty() { "" } else { "." }),
        (if column.is_empty() { "" } else { "." }),
        source.message
    ))]
  GraphqlDynamicPermissionError {
    #[snafu(source(false))]
    source: Box<async_graphql::Error>,
    field: String,
    column: String,
    context_path: String,
  },
}

impl AuthError {
  pub fn from_graphql_dynamic_subscribe_id_guard(source: async_graphql::Error, context: &ResolverContext, field_name: &str, column_name: &str) -> AuthError {
    AuthError::GraphqlDynamicPermissionError {
      source: Box::new(source),
      field: field_name.to_string(),
      column: column_name.to_string(),
      context_path: context.ctx.path_node.map(|p| p.to_string_vec().join("")).unwrap_or_default(),
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthErrorResponse {
  pub success: bool,
  pub message: String,
}

impl From<AuthError> for AuthErrorResponse {
  fn from(value: AuthError) -> Self {
    AuthErrorResponse {
      success: false,
      message: value.to_string(),
    }
  }
}

impl IntoResponse for AuthError {
  fn into_response(self) -> Response {
    (
      match &self {
        Self::SessionConfiguration { .. } | Self::SessionStorage => StatusCode::INTERNAL_SERVER_ERROR,
        Self::PermissionError | Self::GraphqlDynamicPermissionError { .. } => StatusCode::FORBIDDEN,
        _ => StatusCode::UNAUTHORIZED,
      },
      Json(AuthErrorResponse::from(self)),
    )
      .into_response()
  }
}
