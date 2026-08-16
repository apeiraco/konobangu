//! `ETag` Middleware for Caching Requests
//!
//! This middleware implements the [ETag](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag)
//! HTTP header for caching responses in Axum. `ETags` are used to validate
//! cache entries by comparing a client's stored `ETag` with the one generated
//! by the server. If the `ETags` match, a `304 Not Modified` response is sent,
//! avoiding the need to resend the full content.

use std::{
  sync::Arc,
  task::{Context, Poll},
};

use axum::{Router, body::Body, extract::Request, http::StatusCode, response::Response};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tower::{Layer, Service};

use crate::{app::AppContextTrait, errors::RecorderResult, web::middleware::MiddlewareLayer};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Etag {
  #[serde(default)]
  pub enable: bool,
}

impl MiddlewareLayer for Etag {
  /// Returns the name of the middleware
  fn name(&self) -> &'static str {
    "etag"
  }

  /// Returns whether the middleware is enabled or not
  fn is_enabled(&self) -> bool {
    self.enable
  }

  fn config(&self) -> serde_json::Result<serde_json::Value> {
    serde_json::to_value(self)
  }

  /// Applies the `ETag` middleware to the application router.
  fn apply(&self, app: Router<Arc<dyn AppContextTrait>>) -> RecorderResult<Router<Arc<dyn AppContextTrait>>> {
    Ok(app.layer(EtagLayer))
  }
}

/// [`EtagLayer`] struct for adding `ETag` functionality as a Tower service
/// layer.
#[derive(Default, Clone)]
struct EtagLayer;

impl<S> Layer<S> for EtagLayer {
  type Service = EtagMiddleware<S>;

  fn layer(&self, inner: S) -> Self::Service {
    EtagMiddleware { inner }
  }
}

#[derive(Clone)]
struct EtagMiddleware<S> {
  inner: S,
}

impl<S> Service<Request<Body>> for EtagMiddleware<S>
where
  S: Service<Request, Response = Response> + Send + 'static,
  S::Future: Send + 'static,
{
  type Response = S::Response;
  type Error = S::Error;
  // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
  type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.inner.poll_ready(cx)
  }

  fn call(&mut self, request: Request) -> Self::Future {
    use axum_extra::headers::{ETag, HeaderMapExt, IfNoneMatch};
    let eligible = matches!(*request.method(), http::Method::GET | http::Method::HEAD);
    let ifnm = request.headers().typed_get::<IfNoneMatch>();

    let future = self.inner.call(request);

    let res_fut = async move {
      let mut response = future.await?;
      if eligible
        && response.status().is_success()
        && let Some(condition) = ifnm
        && (condition == IfNoneMatch::any() || response.headers().typed_get::<ETag>().is_some_and(|etag| !condition.precondition_passes(&etag)))
      {
        *response.status_mut() = StatusCode::NOT_MODIFIED;
        *response.body_mut() = crate::storage::revalidation_body();
        response.headers_mut().remove(http::header::CONTENT_RANGE);
        response.headers_mut().remove(http::header::CONTENT_LENGTH);
        response.headers_mut().remove(http::header::TRANSFER_ENCODING);
      }
      Ok(response)
    };
    Box::pin(res_fut)
  }
}

#[cfg(test)]
mod tests {
  use axum::routing::{get, post};
  use tower::ServiceExt;

  use super::*;
  #[tokio::test]
  async fn etag_preserves_revalidation_headers_and_never_changes_denials() {
    for status in [200, 206, 401, 403, 404, 416] {
      let router = Router::new()
        .route(
          "/",
          get(move || async move {
            Response::builder()
              .status(status)
              .header("ETag", "\"variant\"")
              .header("Vary", "Origin, Accept")
              .header("Cache-Control", "private, no-cache")
              .header("Content-Range", "bytes 0-2/4")
              .header("Content-Length", "3")
              .body(Body::from("abc"))
              .unwrap()
          }),
        )
        .layer(EtagLayer);
      let response = router
        .oneshot(
          Request::builder()
            .uri("/")
            .header("If-None-Match", "\"other\", W/\"variant\"")
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
      if status < 300 {
        assert_eq!(response.status(), 304);
        assert_eq!(response.headers()["Vary"], "Origin, Accept");
        assert_eq!(response.headers()["Cache-Control"], "private, no-cache");
        assert!(!response.headers().contains_key("Content-Range"));
        assert!(!response.headers().contains_key("Content-Length"));
      } else {
        assert_eq!(response.status().as_u16(), status);
      }
    }
    let router = Router::new()
      .route(
        "/",
        post(|| async { Response::builder().header("ETag", "\"variant\"").body(Body::from("body")).unwrap() }),
      )
      .layer(EtagLayer);
    let response = router
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/")
          .header("If-None-Match", "*")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), 200);
  }
}
