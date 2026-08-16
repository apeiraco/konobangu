use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use crate::{app::AppContextTrait, errors::RecorderResult, web::controller::Controller};

pub const CONTROLLER_PREFIX: &str = "/api/metadata";

#[derive(Serialize)]
pub struct StandardResponse {
  pub success: bool,
  pub message: String,
}

async fn health(State(ctx): State<Arc<dyn AppContextTrait>>) -> RecorderResult<Json<StandardResponse>> {
  ctx
    .db()
    .ping()
    .await
    .inspect_err(|err| tracing::error!(err.msg = %err, err.detail = ?err, "health check database ping error"))?;

  Ok(Json(StandardResponse {
    success: true,
    message: "ok".to_string(),
  }))
}

async fn ping() -> Json<StandardResponse> {
  Json(StandardResponse {
    success: true,
    message: "ok".to_string(),
  })
}

pub async fn create(_context: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
  let router = Router::<Arc<dyn AppContextTrait>>::new()
    .route("/health", get(health))
    .route("/ping", get(ping));

  Ok(Controller::from_nest_router(CONTROLLER_PREFIX, router))
}
