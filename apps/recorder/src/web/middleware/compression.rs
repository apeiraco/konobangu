//! Compression Middleware for Axum
//!
//! This middleware applies compression to HTTP responses to reduce the size of
//! the data being transmitted. This can improve performance by decreasing load
//! times and reducing bandwidth usage. The middleware configuration allows for
//! enabling or disabling compression based on the application settings.

use std::sync::Arc;

use axum::Router;
use serde::{Deserialize, Serialize};
use tower_http::compression::CompressionLayer;

use crate::{app::AppContextTrait, errors::RecorderResult, web::middleware::MiddlewareLayer};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Compression {
  #[serde(default)]
  pub enable: bool,
}

impl MiddlewareLayer for Compression {
  /// Returns the name of the middleware
  fn name(&self) -> &'static str {
    "compression"
  }

  /// Returns whether the middleware is enabled or not
  fn is_enabled(&self) -> bool {
    self.enable
  }

  fn config(&self) -> serde_json::Result<serde_json::Value> {
    serde_json::to_value(self)
  }

  /// Applies the Compression middleware layer to the Axum router.
  fn apply(&self, app: Router<Arc<dyn AppContextTrait>>) -> RecorderResult<Router<Arc<dyn AppContextTrait>>> {
    Ok(app.layer(CompressionLayer::new()))
  }
}

#[cfg(test)]
mod tests {
  use axum::{body::Body, http::Request, response::Response, routing::get};
  use tower::ServiceExt;

  use super::*;
  #[tokio::test]
  async fn media_formats_are_never_recompressed() {
    let context = Arc::new(crate::test_utils::app::TestingAppContext::builder().build()) as Arc<dyn AppContextTrait>;
    for mime in ["image/jxl", "image/webp", "image/avif"] {
      let app = Router::<Arc<dyn AppContextTrait>>::new().route(
        "/",
        get(move || async move { Response::builder().header("Content-Type", mime).body(Body::from(vec![0u8; 4096])).unwrap() }),
      );
      let app = Compression { enable: true }.apply(app).unwrap().with_state(context.clone());
      let response = app
        .oneshot(Request::builder().uri("/").header("Accept-Encoding", "br,gzip").body(Body::empty()).unwrap())
        .await
        .unwrap();
      assert_eq!(response.status(), 200);
      assert!(!response.headers().contains_key("Content-Encoding"));
      assert_eq!(axum::body::to_bytes(response.into_body(), 8192).await.unwrap().len(), 4096);
    }
  }
}
