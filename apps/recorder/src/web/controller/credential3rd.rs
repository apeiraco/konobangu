use std::sync::Arc;

use axum::{
  Extension, Json, Router,
  extract::{Path, State},
  http::{HeaderMap, StatusCode},
  middleware::from_fn_with_state,
  response::{IntoResponse, Response},
  routing::post,
};
use sea_orm::{
  ActiveValue::Set,
  ColumnTrait, ConnectionTrait, EntityTrait, FromQueryResult, QueryFilter, QuerySelect, QueryTrait,
  sea_query::{Expr, ExprTrait},
};
use serde::Serialize;

use crate::{
  app::AppContextTrait,
  auth::{AuthError, AuthUserInfo, auth_middleware, session::csrf::validate_cookie_write},
  database::operation::begin_task_transaction,
  errors::{RecorderError, RecorderResult},
  models::credential_3rd::{self, Column, Entity, Model},
  web::controller::Controller,
};

#[derive(Serialize)]
struct CheckResult {
  available: bool,
}

async fn check(ctx: &dyn AppContextTrait, id: i32, subscriber_id: i32) -> RecorderResult<CheckResult> {
  let read = begin_task_transaction(ctx.db().as_ref(), subscriber_id).await?;
  let statement = Entity::find_by_id(id)
    .filter(Column::SubscriberId.eq(subscriber_id))
    .column_as(Expr::cust("xmin::text"), "credential_version")
    .build(read.get_database_backend());
  let row = read.query_one_raw(statement).await?.ok_or(AuthError::PermissionError)?;
  let snapshot = Model::from_query_result(&row, "")?;
  let version: String = row.try_get("", "credential_version")?;
  read.rollback().await?;

  // External authentication owns no business connection. The MVCC version is
  // captured with the row, so even an ABA edit cannot accept stale cookies.
  let (available, cookies) = snapshot
    .clone()
    .check_remote_availability(ctx)
    .await
    .map_err(|_| RecorderError::from_status(StatusCode::BAD_GATEWAY))?;
  let cookies = cookies.map(|cookies| ctx.crypto().encrypt_string(cookies)).transpose()?.or(snapshot.cookies);
  let write = begin_task_transaction(ctx.db().as_ref(), subscriber_id).await?;
  let updated = Entity::update_many()
    .set(credential_3rd::ActiveModel {
      cookies: Set(cookies),
      updated_at: Set(chrono::Utc::now()),
      ..Default::default()
    })
    .filter(Column::Id.eq(id))
    .filter(Column::SubscriberId.eq(subscriber_id))
    .filter(Expr::cust("xmin::text").eq(version))
    .exec(&write)
    .await?;
  if updated.rows_affected != 1 {
    write.rollback().await?;
    return Err(RecorderError::from_status(StatusCode::CONFLICT));
  }
  write.commit().await?;
  Ok(CheckResult { available })
}

async fn check_available(
  State(ctx): State<Arc<dyn AppContextTrait>>,
  Path(id): Path<i32>,
  Extension(user): Extension<AuthUserInfo>,
  headers: HeaderMap,
  Json(_body): Json<serde_json::Map<String, serde_json::Value>>,
) -> Response {
  if let Err(status) = validate_cookie_write(&headers, &ctx.auth().runtime().config.external_base_url) {
    return status.into_response();
  }
  let response = match check(ctx.as_ref(), id, user.subscriber_auth.subscriber_id).await {
    Ok(result) => Json(result).into_response(),
    Err(error) => error.into_response(),
  };
  let (mut parts, body) = response.into_parts();
  parts.headers.insert(axum::http::header::CACHE_CONTROL, "no-store".parse().unwrap());
  Response::from_parts(parts, body)
}

pub async fn create(ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
  let router = Router::<Arc<dyn AppContextTrait>>::new().route("/{id}/check-available", post(check_available).layer(from_fn_with_state(ctx, auth_middleware)));
  Ok(Controller::from_nest_router("/api/credential3rd", router))
}
