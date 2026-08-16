use std::sync::Arc;

use axum::{
  Extension, Router,
  body::Body,
  extract::{Path, Query, State},
  middleware::from_fn_with_state,
  response::Response,
  routing::get,
};
use axum_extra::headers::Header;
use headers_accept::Accept;
use http::{HeaderMap, Method, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{
  app::AppContextTrait,
  auth::{AuthError, AuthUserInfo, auth_middleware},
  errors::RecorderResult,
  web::controller::Controller,
};

pub const CONTROLLER_PREFIX: &str = "/api/static";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum OptimizeType {
  #[serde(rename = "accept")]
  AcceptHeader,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StaticQuery {
  optimize: Option<OptimizeType>,
}

async fn serve_subscriber_static(
  State(ctx): State<Arc<dyn AppContextTrait>>,
  Path((subscriber_id, path)): Path<(i32, String)>,
  Extension(auth_user_info): Extension<AuthUserInfo>,
  Query(query): Query<StaticQuery>,
  method: Method,
  headers: HeaderMap,
) -> RecorderResult<Response> {
  if subscriber_id != auth_user_info.subscriber_auth.subscriber_id {
    Err(AuthError::PermissionError)?;
  }
  if std::path::Path::new(&path)
    .components()
    .any(|part| !matches!(part, std::path::Component::Normal(_)))
  {
    return Err(AuthError::PermissionError.into());
  }
  let storage = ctx.storage();

  let storage_path = storage.build_subscriber_path(subscriber_id, &path);

  deliver(ctx.as_ref(), &method, storage_path.as_str(), &headers, &query, true).await
}

async fn serve_public_static(
  State(ctx): State<Arc<dyn AppContextTrait>>,
  Path(path): Path<String>,
  Query(query): Query<StaticQuery>,
  method: Method,
  headers: HeaderMap,
) -> RecorderResult<Response> {
  if std::path::Path::new(&path)
    .components()
    .any(|part| !matches!(part, std::path::Component::Normal(_)))
  {
    return Err(AuthError::PermissionError.into());
  }
  let storage = ctx.storage();

  let storage_path = storage.build_public_path(&path);

  deliver(ctx.as_ref(), &method, storage_path.as_str(), &headers, &query, false).await
}

async fn deliver(ctx: &dyn AppContextTrait, method: &Method, path: &str, headers: &HeaderMap, query: &StaticQuery, private: bool) -> RecorderResult<Response> {
  let optimized = query.optimize.is_some();
  let accept = if headers.contains_key(http::header::ACCEPT) {
    let mut values = headers.get_all(http::header::ACCEPT).iter();
    match Accept::decode(&mut values) {
      Ok(value) if crate::media::negotiation::validate_accept(&value).is_ok() => Some(value),
      _ => {
        let mut response = Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from("Invalid Accept header"))?;
        if optimized {
          crate::storage::merge_vary(response.headers_mut(), "Accept");
        }
        return Ok(response);
      }
    }
  } else {
    None
  };
  if optimized {
    ctx.storage().serve_image(method, path, headers, accept.as_ref(), ctx.media(), private).await
  } else {
    let mut response = ctx.storage().serve_representation(method, path, headers, None).await?;
    response.headers_mut().insert(
      http::header::CACHE_CONTROL,
      http::HeaderValue::from_static(if private { "private, no-cache" } else { "public, no-cache" }),
    );
    Ok(response)
  }
}

pub async fn create(ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
  let router = Router::<Arc<dyn AppContextTrait>>::new()
    .route(
      "/subscribers/{subscriber_id}/{*path}",
      get(serve_subscriber_static).layer(from_fn_with_state(ctx, auth_middleware)),
    )
    .route("/public/{*path}", get(serve_public_static));

  Ok(Controller::from_nest_router(CONTROLLER_PREFIX, router))
}
