use std::sync::Arc;

use axum::{
  extract::{Request, State},
  http::header,
  middleware::Next,
  response::{IntoResponse, Response},
};

use crate::{
  app::AppContextTrait,
  auth::{AuthService, AuthServiceTrait},
};

pub async fn auth_middleware(State(ctx): State<Arc<dyn AppContextTrait>>, request: Request, next: Next) -> Response {
  let auth_service = ctx.auth();

  let (mut parts, body) = request.into_parts();

  let mut response = match auth_service.extract_user_info(ctx.as_ref() as &dyn AppContextTrait, &mut parts).await {
    Ok(auth_user_info) => {
      let mut request = Request::from_parts(parts, body);
      request.extensions_mut().insert(auth_user_info);
      next.run(request).await
    }
    Err(auth_error) => auth_error.into_response(),
  };

  if let Some(header_value) = auth_service.www_authenticate_header_value() {
    response.headers_mut().insert(header::WWW_AUTHENTICATE, header_value);
  };

  response
}

pub async fn webui_auth_middleware(State(ctx): State<Arc<dyn AppContextTrait>>, request: Request, next: Next) -> Response {
  if (!request.uri().path().starts_with("/api"))
    && let AuthService::Basic(auth_service) = ctx.auth()
  {
    let (mut parts, body) = request.into_parts();

    let mut response = match auth_service.extract_user_info(ctx.as_ref() as &dyn AppContextTrait, &mut parts).await {
      Ok(auth_user_info) => {
        let mut request = Request::from_parts(parts, body);
        request.extensions_mut().insert(auth_user_info);
        next.run(request).await
      }
      Err(auth_error) => auth_error.into_response(),
    };

    if let Some(header_value) = auth_service.www_authenticate_header_value() {
      response.headers_mut().insert(header::WWW_AUTHENTICATE, header_value);
    };

    response
  } else {
    next.run(request).await
  }
}
