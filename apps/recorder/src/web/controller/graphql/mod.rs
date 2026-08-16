use std::sync::Arc;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{Extension, Router, extract::State, middleware::from_fn_with_state, routing::post};

use super::core::Controller;
use crate::{
  app::{AppContextTrait, Environment},
  auth::{AuthUserInfo, auth_middleware},
  errors::RecorderResult,
};

pub const CONTROLLER_PREFIX: &str = "/api/graphql";

async fn graphql_handler(
  State(ctx): State<Arc<dyn AppContextTrait>>,
  Extension(auth_user_info): Extension<AuthUserInfo>,
  headers: axum::http::HeaderMap,
  req: GraphQLRequest,
) -> axum::response::Response {
  use axum::response::IntoResponse;
  let mut request = req.into_inner();
  if matches!(
    crate::graphql::operation::operation_type(&mut request),
    Ok(async_graphql::parser::types::OperationType::Mutation)
  ) && let Err(status) = crate::auth::session::csrf::validate_cookie_write(&headers, &ctx.auth().runtime().config.external_base_url)
  {
    let response = GraphQLResponse::from(async_graphql::Response::from_errors(vec![async_graphql::ServerError::new(
      "Cookie write validation failed",
      None,
    )]));
    return (status, response).into_response();
  }
  GraphQLResponse::from(crate::graphql::operation::execute_operation(&ctx.graphql().schema, ctx.db().as_ref(), auth_user_info, request).await).into_response()
}

async fn graphql_introspection_handler(State(ctx): State<Arc<dyn AppContextTrait>>, req: GraphQLRequest) -> GraphQLResponse {
  ctx.graphql().schema.execute(req.into_inner().only_introspection()).await.into()
}

pub async fn create(ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
  let mut introspection_handler = post(graphql_introspection_handler);

  if !matches!(ctx.environment(), Environment::Development) {
    introspection_handler = introspection_handler.layer(from_fn_with_state(ctx.clone(), auth_middleware));
  }

  let router = Router::<Arc<dyn AppContextTrait>>::new()
    .route("/", post(graphql_handler).layer(from_fn_with_state(ctx, auth_middleware)))
    .route("/introspection", introspection_handler);
  Ok(Controller::from_nest_router(CONTROLLER_PREFIX, router))
}
