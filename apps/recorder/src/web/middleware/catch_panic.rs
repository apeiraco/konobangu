//! Catch Panic Middleware for Axum
//!
//! This middleware catches panics that occur during request handling in the
//! application. When a panic occurs, it logs the error and returns an
//! internal server error response. This middleware helps ensure that the
//! application can gracefully handle unexpected errors without crashing the
//! server.
use std::sync::Arc;

use axum::{Router, response::IntoResponse};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tower_http::catch_panic::CatchPanicLayer;

use crate::{app::AppContextTrait, errors::RecorderResult, web::middleware::MiddlewareLayer};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CatchPanic {
  #[serde(default)]
  pub enable: bool,
}

/// Handler function for the [`CatchPanicLayer`] middleware.
///
/// This function processes panics by extracting error messages, logging them,
/// and returning an internal server error response.
#[allow(clippy::needless_pass_by_value)]
fn handle_panic(err: Box<dyn std::any::Any + Send + 'static>) -> axum::response::Response {
  let err = err
    .downcast_ref::<String>()
    .map_or_else(|| err.downcast_ref::<&str>().map_or("no error details", |s| s), |s| s.as_str());

  tracing::error!(err.msg = err, "server_panic");

  StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

impl MiddlewareLayer for CatchPanic {
  /// Returns the name of the middleware
  fn name(&self) -> &'static str {
    "catch_panic"
  }

  /// Returns whether the middleware is enabled or not
  fn is_enabled(&self) -> bool {
    self.enable
  }

  fn config(&self) -> serde_json::Result<serde_json::Value> {
    serde_json::to_value(self)
  }

  /// Applies the Catch Panic middleware layer to the Axum router.
  fn apply(&self, app: Router<Arc<dyn AppContextTrait>>) -> RecorderResult<Router<Arc<dyn AppContextTrait>>> {
    Ok(app.layer(CatchPanicLayer::custom(handle_panic)))
  }
}
