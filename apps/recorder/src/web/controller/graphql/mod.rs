use std::sync::Arc;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{Extension, Router, extract::State, middleware::from_fn_with_state, routing::post};
use sea_orm::TransactionTrait;

use super::core::Controller;
use crate::{
    app::{AppContextTrait, Environment},
    auth::{AuthUserInfo, auth_middleware},
    database::rls::bind_subscriber_to_transaction,
    errors::RecorderResult,
};

pub const CONTROLLER_PREFIX: &str = "/api/graphql";

async fn graphql_handler(
    State(ctx): State<Arc<dyn AppContextTrait>>,
    Extension(auth_user_info): Extension<AuthUserInfo>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let graphql_service = ctx.graphql();
    let subscriber_id = auth_user_info.subscriber_auth.subscriber_id;

    // Start a transaction and bind the subscriber identity for RLS.
    // `SET LOCAL` scopes the variable to this transaction only, preventing
    // state leakage across connection pool reuse.
    let txn = match ctx.db().begin().await {
        Ok(txn) => txn,
        Err(e) => {
            tracing::error!(error = ?e, "Failed to begin transaction for RLS binding");
            return async_graphql::Response::from_errors(vec![
                async_graphql::ServerError::new(
                    "Internal server error: failed to initialize database transaction",
                    None,
                ),
            ])
            .into();
        }
    };

    if let Err(e) = bind_subscriber_to_transaction(&txn, subscriber_id).await {
        tracing::error!(error = ?e, subscriber_id, "Failed to bind subscriber to transaction");
        return async_graphql::Response::from_errors(vec![
            async_graphql::ServerError::new(
                "Internal server error: failed to bind subscriber identity",
                None,
            ),
        ])
        .into();
    }

    let mut req = req.into_inner();
    req = req.data(auth_user_info);
    // Inject the transaction so custom mutations can use it via
    // `ctx.data::<DatabaseTransaction>()` to benefit from RLS.
    req = req.data(txn);

    let response = graphql_service.schema.execute(req).await;

    // Retrieve the transaction back from the request context to commit it.
    // Note: The transaction is moved into the request data, so we cannot
    // directly access it here. Instead, we rely on the Drop impl of
    // DatabaseTransaction which will roll back if not committed.
    // For read-only queries this is fine. For mutations that use the
    // injected transaction, they should commit within their own scope.
    //
    // Since seaography's standard mutations use the global DatabaseConnection
    // (not this transaction), the transaction primarily serves as the RLS
    // binding context. It will be rolled back on drop, which is safe.

    response.into()
}

// 检查是否是 introspection 查询
fn is_introspection_query(req: &async_graphql::Request) -> bool {
    if let Some(operation) = &req.operation_name
        && operation.starts_with("__")
    {
        return true;
    }

    // 检查查询内容是否包含 introspection 字段
    let query = req.query.as_str();
    query.contains("__schema") || query.contains("__type") || query.contains("__typename")
}

async fn graphql_introspection_handler(
    State(ctx): State<Arc<dyn AppContextTrait>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let graphql_service = ctx.graphql();
    let req = req.into_inner();

    if !is_introspection_query(&req) {
        return GraphQLResponse::from(async_graphql::Response::from_errors(vec![
            async_graphql::ServerError::new(
                "Only introspection queries are allowed on this endpoint",
                None,
            ),
        ]));
    }

    graphql_service.schema.execute(req).await.into()
}

pub async fn create(ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
    let mut introspection_handler = post(graphql_introspection_handler);

    if !matches!(ctx.environment(), Environment::Development) {
        introspection_handler =
            introspection_handler.layer(from_fn_with_state(ctx.clone(), auth_middleware));
    }

    let router = Router::<Arc<dyn AppContextTrait>>::new()
        .route(
            "/",
            post(graphql_handler).layer(from_fn_with_state(ctx, auth_middleware)),
        )
        .route("/introspection", introspection_handler);
    Ok(Controller::from_nest_router(CONTROLLER_PREFIX, router))
}
