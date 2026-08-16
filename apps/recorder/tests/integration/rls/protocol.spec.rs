use recorder::auth::{
  config::{AuthSessionConfig, OidcAuthConfig},
  oidc::OidcAuthService,
  session::{
    pending::{PendingStoreConfig, PostgresPendingStore},
    runtime::SessionRuntime,
  },
};
use securitydept_oidc_client::{OidcCodeCallbackSearchParams, PendingOauthStore};

use super::support::RlsFixture;

#[tokio::test]
async fn published_sdk_rejects_invalid_signed_tokens_and_access_restrictions() {
  let fixture = RlsFixture::new().await;
  let mut server = mockito::Server::new_async().await;
  let issuer = server.url();
  let generator = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../webui/tests/integration/auth/negative-tokens.mts");
  let output = std::process::Command::new("node")
    .arg(generator)
    .arg(&issuer)
    .output()
    .expect("mise Node and installed jose are required");
  assert!(output.status.success(), "jose fixture generation failed");
  let generated: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
  let metadata = serde_json::json!({ "issuer": issuer, "authorization_endpoint": format!("{issuer}/authorize"), "token_endpoint": format!("{issuer}/token"), "jwks_uri": format!("{issuer}/jwks"), "response_types_supported": ["code"], "subject_types_supported": ["public"], "id_token_signing_alg_values_supported": ["RS256"], "token_endpoint_auth_methods_supported": ["client_secret_basic"], "code_challenge_methods_supported": ["S256"] });
  let _discovery = server
    .mock("GET", "/.well-known/openid-configuration")
    .with_header("content-type", "application/json")
    .with_body(metadata.to_string())
    .create_async()
    .await;
  let _jwks = server
    .mock("GET", "/jwks")
    .with_header("content-type", "application/json")
    .with_body(generated["jwks"].to_string())
    .create_async()
    .await;
  for scenario in ["nonce", "issuer", "audience", "expired", "signature", "scope", "pkce"] {
    let common: AuthSessionConfig = serde_json::from_value(
      serde_json::json!({ "identity_database_uri": fixture.identity_uri, "external_base_url": "http://127.0.0.1:5001", "session_cookie_secure": false }),
    )
    .unwrap();
    let runtime = SessionRuntime::new(common.clone()).await.unwrap();
    let config: OidcAuthConfig = serde_json::from_value(serde_json::json!({ "oidc_issuer": issuer, "oidc_audience": "konobangu-test", "oidc_client_id": "konobangu-test", "oidc_client_secret": "negative-test-secret", "oidc_extra_scopes": if scenario == "scope" { vec!["restricted"] } else { vec![] }, "identity_database_uri": common.identity_database_uri, "external_base_url": common.external_base_url, "session_cookie_secure": false })).unwrap();
    let service = OidcAuthService::new(config, runtime).await.unwrap();
    let authorization = service
      .client
      .handle_code_authorize_with_redirect_override_and_extra_data(&common.external_base_url, None, None)
      .await
      .unwrap();
    let pending = PostgresPendingStore::from_config(&PendingStoreConfig {
      pool: Some(service.runtime.identity_db.get_postgres_connection_pool().clone()),
      ttl_seconds: 300,
    });
    let state = authorization.csrf_token.secret().clone();
    let request = pending.take(&state).await.unwrap().unwrap();
    pending.insert(state.clone(), "bound-nonce".into(), request.code_verifier, None).await.unwrap();
    let body = if scenario == "pkce" {
      serde_json::json!({"error":"invalid_grant", "error_description":"PKCE verification failed"})
    } else {
      serde_json::json!({ "access_token": "invalid-fixture-only", "token_type": "Bearer", "scope": "openid profile", "id_token": generated["tokens"][scenario] })
    };
    let token = server
      .mock("POST", "/token")
      .match_body(mockito::Matcher::Regex("code_verifier=".into()))
      .with_status(if scenario == "pkce" { 400 } else { 200 })
      .with_header("content-type", "application/json")
      .with_body(body.to_string())
      .expect(1)
      .create_async()
      .await;
    let result = service
      .client
      .handle_code_callback(
        OidcCodeCallbackSearchParams {
          code: "negative-only".into(),
          state: Some(state.clone()),
        },
        &common.external_base_url,
      )
      .await;
    assert!(result.is_err(), "{scenario} must be rejected by the published SDK verifier");
    assert!(pending.take(&state).await.unwrap().is_none(), "a failed callback cannot be retried");
    token.assert_async().await;
    token.remove_async().await;
    service.runtime.close().await.unwrap();
  }
  fixture.close().await;
}
