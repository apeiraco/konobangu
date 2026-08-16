use std::{collections::HashMap, sync::Arc};

use axum::{
  Extension, Json, Router,
  extract::{Query, Request, State},
  http::{StatusCode, header},
  middleware::{Next, from_fn, from_fn_with_state},
  response::{IntoResponse, Redirect, Response},
  routing::{get, post},
};
use securitydept_oidc_client::OidcCodeCallbackSearchParams;
use securitydept_session_context::{SessionContext, SessionContextSession, SessionPrincipal};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::{
  app::AppContextTrait,
  auth::{
    AuthError, AuthService, AuthUserInfo, auth_middleware,
    session::{
      csrf::{validate_cookie_write, validate_return_path},
      runtime::GRANT_KEY,
    },
  },
  errors::RecorderResult,
  models::auth,
  web::controller::core::Controller,
};

pub const CONTROLLER_PREFIX: &str = "/api/auth/session";
const BINDINGS_KEY: &str = "konobangu.pending_logins";
const MAX_PENDING_LOGINS: usize = 8;

#[derive(Deserialize, Default)]
struct LoginQuery {
  #[serde(default, rename = "post_auth_redirect_uri")]
  return_path: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
struct BrowserLogin {
  state: String,
  return_path: String,
}

#[allow(
  clippy::result_large_err,
  reason = "Axum handlers return full Response errors to preserve HTTP status, headers and body without changing the framework contract"
)]
async fn login(State(ctx): State<Arc<dyn AppContextTrait>>, session: Session, Query(query): Query<LoginQuery>) -> Result<Redirect, Response> {
  let AuthService::Oidc(service) = ctx.auth() else {
    return Err(StatusCode::BAD_REQUEST.into_response());
  };
  let return_path =
    validate_return_path(query.return_path.as_deref().unwrap_or("/"), &service.runtime.config.external_base_url).map_err(IntoResponse::into_response)?;
  let mut bindings = session
    .get::<Vec<BrowserLogin>>(BINDINGS_KEY)
    .await
    .map_err(|_| AuthError::SessionStorage.into_response())?
    .unwrap_or_default();
  if bindings.len() >= MAX_PENDING_LOGINS {
    bindings.remove(0);
  }
  let authorization = service
    .client
    .handle_code_authorize_with_redirect_override_and_extra_data(&service.runtime.config.external_base_url, None, None)
    .await
    .map_err(|_| AuthError::SessionStorage.into_response())?;
  bindings.push(BrowserLogin {
    state: authorization.csrf_token.secret().clone(),
    return_path,
  });
  session
    .insert(BINDINGS_KEY, bindings)
    .await
    .map_err(|_| AuthError::SessionStorage.into_response())?;
  Ok(Redirect::to(authorization.authorization_url.as_str()))
}

#[allow(
  clippy::result_large_err,
  reason = "Axum handlers return full Response errors to preserve HTTP status, headers and body without changing the framework contract"
)]
async fn callback(
  State(ctx): State<Arc<dyn AppContextTrait>>,
  session: Session,
  Query(query): Query<OidcCodeCallbackSearchParams>,
) -> Result<Redirect, Response> {
  let AuthService::Oidc(service) = ctx.auth() else {
    return Err(StatusCode::BAD_REQUEST.into_response());
  };
  let mut bindings = session
    .get::<Vec<BrowserLogin>>(BINDINGS_KEY)
    .await
    .map_err(|_| AuthError::SessionStorage.into_response())?
    .unwrap_or_default();
  let index = bindings
    .iter()
    .position(|binding| Some(&binding.state) == query.state.as_ref())
    .ok_or_else(|| AuthError::SessionUnauthorized.into_response())?;
  let binding = bindings.remove(index);
  // Check both browser correlation and return target before the SDK consumes
  // state or we create a grant.
  let return_path = validate_return_path(&binding.return_path, &service.runtime.config.external_base_url).map_err(IntoResponse::into_response)?;
  session
    .insert(BINDINGS_KEY, bindings)
    .await
    .map_err(|_| AuthError::SessionStorage.into_response())?;
  let result = service
    .client
    .handle_code_callback(query, &service.runtime.config.external_base_url)
    .await
    .map_err(|_| AuthError::SessionUnauthorized.into_response())?;
  let issuer = result.id_token_claims.issuer().to_string();
  let subject = result.id_token_claims.subject().to_string();
  let identity = auth::Model::find_or_create_oidc(&service.runtime.identity_db, &issuer, &subject)
    .await
    .map_err(|_| AuthError::SessionStorage.into_response())?;
  let principal = SessionPrincipal::builder()
    .subject(subject.clone())
    .display_name(subject)
    .issuer(issuer)
    .build();
  // Reauthentication revokes the previous local grant, including concurrent
  // late saves.
  service.runtime.revoke(&session).await.map_err(IntoResponse::into_response)?;
  session.cycle_id().await.map_err(|_| AuthError::SessionStorage.into_response())?;
  let grant_id = service.runtime.create_grant(identity.id).await.map_err(IntoResponse::into_response)?;
  SessionContextSession::from_resolved_config(session.clone(), &service.runtime.sdk_config)
    .insert(&SessionContext::<HashMap<String, serde_json::Value>>::builder().principal(principal).build())
    .await
    .map_err(|_| AuthError::SessionStorage.into_response())?;
  session
    .insert(GRANT_KEY, grant_id)
    .await
    .map_err(|_| AuthError::SessionStorage.into_response())?;
  Ok(Redirect::to(&return_path))
}

async fn user_info(
  State(ctx): State<Arc<dyn AppContextTrait>>,
  Extension(user): Extension<AuthUserInfo>,
  session: Session,
) -> Result<Json<SessionContext>, AuthError> {
  if matches!(ctx.auth(), AuthService::Basic(_)) {
    return Ok(Json(
      SessionContext::builder()
        .principal(
          SessionPrincipal::builder()
            .subject(user.subscriber_auth.pid)
            .display_name("Administrator")
            .build(),
        )
        .build(),
    ));
  }
  // Authentication middleware already checked the persistent revocation
  // barrier.
  let context = SessionContextSession::from_resolved_config(session, &ctx.auth().runtime().sdk_config)
    .require()
    .await
    .map_err(|_| AuthError::SessionUnauthorized)?;
  Ok(Json(context))
}

async fn logout(State(ctx): State<Arc<dyn AppContextTrait>>, session: Session, headers: axum::http::HeaderMap) -> Response {
  if let Err(status) = validate_cookie_write(&headers, &ctx.auth().runtime().config.external_base_url) {
    return status.into_response();
  }
  if matches!(ctx.auth(), AuthService::Basic(_)) {
    return (
      StatusCode::CONFLICT,
      Json(serde_json::json!({"message": "Browser-cached Basic credentials cannot be revoked by clearing a session cookie", "logout_supported": false})),
    )
      .into_response();
  }
  match ctx.auth().runtime().revoke(&session).await {
    Ok(()) => Json(serde_json::json!({"success": true})).into_response(),
    Err(error) => error.into_response(),
  }
}

async fn no_cache(request: Request, next: Next) -> Response {
  let mut response = next.run(request).await;
  response
    .headers_mut()
    .insert(header::CACHE_CONTROL, header::HeaderValue::from_static("no-store"));
  response
}

pub async fn create(ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
  let router = Router::<Arc<dyn AppContextTrait>>::new()
    .route("/login", get(login))
    .route("/callback", get(callback))
    .route("/user-info", get(user_info).layer(from_fn_with_state(ctx, auth_middleware)))
    .route("/logout", post(logout))
    .layer(from_fn(no_cache));
  Ok(Controller::from_nest_router(CONTROLLER_PREFIX, router))
}
