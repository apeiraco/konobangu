use std::sync::Arc;

use async_graphql::{Request, dynamic::Schema};
use recorder::{
  app::AppContextTrait,
  auth::AuthUserInfo,
  database::DatabaseService,
  graphql::{build_schema, operation::execute_operation},
  models::auth::{AuthType, Model},
  test_utils::{app::TestingAppContext, crypto::build_testing_crypto_service},
};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::Value;

use super::support::{RlsFixture, SUBSCRIBER_A, SUBSCRIBER_B};

fn user(owner: i32, basic: bool) -> AuthUserInfo {
  let auth_type = if basic { AuthType::Basic } else { AuthType::Oidc };
  AuthUserInfo {
    auth_type: auth_type.clone(),
    subscriber_auth: Model {
      id: owner,
      pid: "shared-owner-fixture".into(),
      issuer: (!basic).then(|| "https://fixture.example".into()),
      subscriber_id: owner,
      auth_type,
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    },
  }
}
async fn gql(schema: &Schema, ctx: &dyn AppContextTrait, basic: bool, document: impl Into<String>) -> Value {
  let response = execute_operation(schema, ctx.db(), user(SUBSCRIBER_A, basic), Request::new(document)).await;
  assert!(response.errors.is_empty(), "{:?}", response.errors);
  response.data.into_json().unwrap()
}

#[tokio::test]
async fn shared_subscriber_oidc_cannot_access_existing_system_crons() {
  let fixture = RlsFixture::new().await;
  let mut config = fixture.bootstrap.config.clone();
  config.uri = fixture.app_uri.clone();
  config.auto_migrate = false;
  let ctx = Arc::new(
    TestingAppContext::builder()
      .db(DatabaseService::from_config(config).await.unwrap())
      .crypto(build_testing_crypto_service().await.unwrap())
      .build(),
  );
  let schema = build_schema(ctx.clone(), None, None).unwrap();
  let system = gql(&schema, ctx.as_ref(), true,
    r#"mutation { cronCreateOne(data: { cronExpr: "0 0 0 * * *", cronTimezone: "UTC", enabled: false, systemTaskCron: { taskType: "test", taskId: "fixture-system" } }) { id } }"#,
  ).await["cronCreateOne"]["id"].as_i64().unwrap();
  let own = gql(&schema, ctx.as_ref(), false,
    r#"mutation { cronCreateOne(data: { cronExpr: "0 0 0 * * *", cronTimezone: "UTC", subscriberTaskCron: { taskType: "sync_one_subscription_sources", subscriptionId: 1 } }) { id } }"#,
  ).await["cronCreateOne"]["id"].as_i64().unwrap();
  // Historic rows can be reached through a subscription relation as well as the
  // root.
  fixture
    .bootstrap
    .execute_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "UPDATE cron SET subscription_id = 1 WHERE id = $1",
      [system.into()],
    ))
    .await
    .unwrap();
  let query = "{ cron { nodes { id } } subscriptions { nodes { cron { nodes { id } } } } }";
  let oidc = gql(&schema, ctx.as_ref(), false, query).await;
  assert_eq!(oidc["cron"]["nodes"], serde_json::json!([{"id": own}]));
  assert_eq!(oidc["subscriptions"]["nodes"][0]["cron"]["nodes"], serde_json::json!([{"id": own}]));
  let admin = gql(&schema, ctx.as_ref(), true, query).await;
  assert_eq!(admin["cron"]["nodes"].as_array().unwrap().len(), 2);
  assert_eq!(admin["subscriptions"]["nodes"][0]["cron"]["nodes"].as_array().unwrap().len(), 2);
  for data in ["enabled: true", "cronExpr: \"0 0 1 * * *\", cronTimezone: \"UTC\"", "priority: 123"] {
    let response = gql(
      &schema,
      ctx.as_ref(),
      false,
      format!("mutation {{ cronUpdate(data: {{ {data} }}, filter: {{ id: {{ eq: {system} }} }}) {{ id }} }}",),
    )
    .await;
    assert_eq!(response["cronUpdate"], serde_json::json!([]));
  }
  let mixed = gql(
    &schema,
    ctx.as_ref(),
    false,
    format!("mutation {{ cronUpdate(data: {{ priority: 17 }}, filter: {{ id: {{ is_in: [{system}, {own}] }} }}) {{ id priority }} }}",),
  )
  .await;
  assert_eq!(mixed["cronUpdate"], serde_json::json!([{"id": own, "priority": 17}]));
  let system_row = gql(
    &schema,
    ctx.as_ref(),
    true,
    format!("{{ cron(filter: {{ id: {{ eq: {system} }} }}) {{ nodes {{ id enabled priority cronExpr }} }} }}",),
  )
  .await;
  assert_eq!(
    system_row["cron"]["nodes"],
    serde_json::json!([{"id": system, "enabled": false, "priority": 0, "cronExpr": "0 0 0 * * *"}])
  );
  let denied = gql(
    &schema,
    ctx.as_ref(),
    false,
    format!("mutation {{ cronDelete(filter: {{ id: {{ eq: {system} }} }}) }}"),
  )
  .await;
  assert_eq!(denied["cronDelete"], 0);
  let mixed_delete = gql(
    &schema,
    ctx.as_ref(),
    false,
    format!("mutation {{ cronDelete(filter: {{ id: {{ is_in: [{system}, {own}] }} }}) }}"),
  )
  .await;
  assert_eq!(mixed_delete["cronDelete"], 1);
  let forbidden = execute_operation(&schema, ctx.db(), user(SUBSCRIBER_A, false), Request::new(
    r#"mutation { cronCreateOne(data: { cronExpr: "0 0 0 * * *", cronTimezone: "UTC", systemTaskCron: { taskType: "test", taskId: "forged-system" } }) { id } }"#,
  )).await;
  assert!(!forbidden.errors.is_empty());
  let other = execute_operation(
    &schema,
    ctx.db(),
    user(SUBSCRIBER_B, true),
    Request::new(format!(
      "mutation {{ cronUpdate(data: {{ enabled: true }}, filter: {{ id: {{ eq: {system} }} }}) {{ id }} }}",
    )),
  )
  .await;
  assert!(other.errors.is_empty());
  assert_eq!(other.data.into_json().unwrap()["cronUpdate"], serde_json::json!([]));
  let updated = gql(
    &schema,
    ctx.as_ref(),
    true,
    format!(
      "mutation {{ cronUpdate(data: {{ enabled: true, cronExpr: \"0 0 2 * * *\", cronTimezone: \"UTC\" }}, filter: {{ id: {{ eq: {system} }} }}) {{ id \
       enabled cronExpr }} }}",
    ),
  )
  .await;
  assert_eq!(
    updated["cronUpdate"],
    serde_json::json!([{"id": system, "enabled": true, "cronExpr": "0 0 2 * * *"}])
  );
  let deleted = gql(
    &schema,
    ctx.as_ref(),
    true,
    format!("mutation {{ cronDelete(filter: {{ id: {{ eq: {system} }} }}) }}"),
  )
  .await;
  assert_eq!(deleted["cronDelete"], 1);
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
}
