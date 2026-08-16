use std::sync::Arc;

use axum::{
  Extension, Router,
  extract::{Path, State},
  middleware::from_fn_with_state,
  response::IntoResponse,
  routing::get,
};
use http::StatusCode;

use crate::{app::AppContextTrait, errors::RecorderResult, models::feeds, web::controller::Controller};

pub const CONTROLLER_PREFIX: &str = "/api/feeds";

async fn rss_handler(
  State(ctx): State<Arc<dyn AppContextTrait>>,
  Path(token): Path<String>,
  Extension(user): Extension<crate::auth::AuthUserInfo>,
) -> RecorderResult<impl IntoResponse> {
  let api_base = ctx.auth().runtime().config.external_base_url.clone();
  let channel = feeds::Model::find_rss_feed_by_token(ctx.as_ref(), &token, &api_base, user.subscriber_auth.subscriber_id)
    .await
    .map_err(|error| match error {
      crate::errors::RecorderError::ModelEntityNotFound { .. } => crate::auth::AuthError::PermissionError.into(),
      error => error,
    })?;

  Ok((StatusCode::OK, [("Content-Type", "application/xml; charset=utf-8")], channel.to_string()))
}

pub async fn create(ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
  let router = Router::<Arc<dyn AppContextTrait>>::new().route("/rss/{token}", get(rss_handler).layer(from_fn_with_state(ctx, crate::auth::auth_middleware)));

  Ok(Controller::from_nest_router(CONTROLLER_PREFIX, router))
}
