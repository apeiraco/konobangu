use std::sync::Arc;

use async_graphql::{Request, Value};
use recorder::{
  auth::AuthUserInfo,
  database::DatabaseService,
  graphql::{build_schema, operation::execute_operation},
  models::auth::{AuthType, Model},
  test_utils::{app::TestingAppContext, crypto::build_testing_crypto_service},
};
use sea_orm::{ConnectionTrait, DbBackend, Statement};

use super::support::{RlsFixture, SUBSCRIBER_A, SUBSCRIBER_B};

fn user(subscriber_id: i32) -> AuthUserInfo {
  AuthUserInfo {
    auth_type: AuthType::Oidc,
    subscriber_auth: Model {
      id: subscriber_id,
      pid: format!("fixture-{subscriber_id}"),
      issuer: Some("https://issuer.example/".into()),
      subscriber_id,
      auth_type: AuthType::Oidc,
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    },
  }
}

#[tokio::test]
async fn production_graphql_isolates_relations_and_commits_or_rolls_back_an_operation() {
  let fixture = RlsFixture::new().await;
  let mut config = fixture.bootstrap.config.clone();
  config.uri = fixture.app_uri.clone();
  config.auto_migrate = false;
  let db = DatabaseService::from_config(config).await.unwrap();
  let ctx = Arc::new(
    TestingAppContext::builder()
      .db(db)
      .crypto(build_testing_crypto_service().await.unwrap())
      .build(),
  );
  let schema = build_schema(ctx.clone(), None, None).unwrap();
  fixture
    .bootstrap
    .execute_unprepared(
      "UPDATE subscriptions SET created_at='2026-10-04T19:38:05.559928Z',updated_at='2026-10-05T03:38:05.559928+08:00' WHERE subscriber_id=1001",
    )
    .await
    .unwrap();
  let timestamps = execute_operation(
    &schema,
    &fixture.app,
    user(SUBSCRIBER_A),
    Request::new("{ subscriptions(filter:{createdAt:{eq:\"2026-10-04 19:38:05.559928\"}}) { nodes { createdAt updatedAt } } }"),
  )
  .await;
  assert!(timestamps.errors.is_empty(), "{:?}", timestamps.errors);
  let timestamps = timestamps.data.into_json().unwrap();
  let stored = fixture
    .bootstrap
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT created_at,updated_at FROM subscriptions WHERE subscriber_id=1001",
    ))
    .await
    .unwrap()
    .unwrap();
  for (field, column) in [("createdAt", "created_at"), ("updatedAt", "updated_at")] {
    let wire = timestamps["subscriptions"]["nodes"][0][field].as_str().unwrap();
    assert!(wire.contains('T') && !wire.contains(" UTC"));
    let persisted: chrono::DateTime<chrono::Utc> = stored.try_get("", column).unwrap();
    assert_eq!(
      chrono::DateTime::parse_from_rfc3339(wire).unwrap().timestamp_micros(),
      persisted.timestamp_micros()
    );
    if field == "createdAt" {
      assert_eq!(persisted.timestamp_micros(), 1_791_142_685_559_928);
    }
  }
  let query = "{ subscriptions { nodes { id displayName subscriber { id displayName } } paginationInfo { total } } subscribers { nodes { id } } }";
  let response_a = execute_operation(&schema, &fixture.app, user(SUBSCRIBER_A), Request::new(query)).await;
  assert!(response_a.errors.is_empty(), "{:?}", response_a.errors);
  let a = response_a.data.into_json().unwrap();
  assert_eq!(a["subscriptions"]["nodes"][0]["displayName"], "A seed");
  assert_eq!(a["subscriptions"]["nodes"][0]["subscriber"]["id"], SUBSCRIBER_A);
  assert_eq!(a["subscribers"]["nodes"].as_array().unwrap().len(), 1);
  assert_eq!(a["subscribers"]["nodes"][0]["id"], SUBSCRIBER_A);
  let response_b = execute_operation(&schema, &fixture.app, user(SUBSCRIBER_B), Request::new(query)).await;
  assert!(response_b.errors.is_empty(), "{:?}", response_b.errors);
  assert_eq!(response_b.data.into_json().unwrap()["subscriptions"]["nodes"][0]["displayName"], "B seed");

  let create = |name: &str| {
    format!(
      r#"subscriptionsCreateOne(data: {{ displayName: "{name}", sourceUrl: "https://example.test/{name}", enabled: true, category: mikan_subscriber }}) {{ id subscriberId displayName }}"#
    )
  };
  let success = execute_operation(
    &schema,
    &fixture.app,
    user(SUBSCRIBER_A),
    Request::new(format!("mutation {{ {} }}", create("committed"))),
  )
  .await;
  assert!(success.errors.is_empty(), "{:?}", success.errors);
  let persisted = fixture
    .bootstrap
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT subscriber_id FROM subscriptions WHERE display_name = 'committed'",
    ))
    .await
    .unwrap()
    .unwrap();
  assert_eq!(persisted.try_get_by_index::<i32>(0).unwrap(), SUBSCRIBER_A);
  let failure = format!(
    r#"mutation {{ first: {} second: subscriptionsCreateOne(data: {{ displayName: "fails", sourceUrl: "https://example.test/fails", enabled: true, category: mikan_subscriber, credentialId: 2147483647 }}) {{ id }} }}"#,
    create("rolled-back")
  );
  let response = execute_operation(&schema, &fixture.app, user(SUBSCRIBER_A), Request::new(failure)).await;
  assert!(!response.errors.is_empty());
  assert_eq!(response.data, Value::Null);
  let count = fixture
    .bootstrap
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT count(*) AS count FROM subscriptions WHERE display_name = 'rolled-back'",
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get_by_index::<i64>(0)
    .unwrap();
  assert_eq!(count, 0, "the first root mutation must roll back with the second");
  let batch = r#"mutation { subscriptionsCreateBatch(data: [
      { displayName: "batch-rolled-back", sourceUrl: "https://example.test/batch", enabled: true, category: mikan_subscriber },
      { displayName: "batch-fails", sourceUrl: "https://example.test/fails", enabled: true, category: mikan_subscriber, credentialId: 2147483647 }
    ]) { id } }"#;
  let failed_batch = execute_operation(&schema, &fixture.app, user(SUBSCRIBER_A), Request::new(batch)).await;
  assert!(!failed_batch.errors.is_empty());
  assert_eq!(failed_batch.data, Value::Null);
  let count = fixture
    .bootstrap
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT count(*) FROM subscriptions WHERE display_name = 'batch-rolled-back'",
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get_by_index::<i64>(0)
    .unwrap();
  assert_eq!(count, 0);
  fixture
    .bootstrap
    .execute_unprepared("INSERT INTO credential3rd (subscriber_id, credential_type, username) VALUES (1002, 'mikan', 'B credential')")
    .await
    .unwrap();
  let hidden_fk = execute_operation(
    &schema,
    &fixture.app,
    user(SUBSCRIBER_A),
    Request::new(format!(
      "mutation {{ {} }}",
      create("hidden-fk").replace("enabled: true", "enabled: true, credentialId: 1")
    )),
  )
  .await;
  assert!(!hidden_fk.errors.is_empty(), "a foreign key must not reference another owner's hidden row");
  let enqueue = |subscription: i32| {
    format!(
      r#"mutation {{ subscriberTasksCreateOne(data: {{ job: {{ taskType: "sync_one_subscription_sources", subscriptionId: {subscription}, subscriberId: 1002 }} }}) {{ id status taskType }} }}"#
    )
  };
  let queued = execute_operation(&schema, &fixture.app, user(SUBSCRIBER_A), Request::new(enqueue(1))).await;
  assert!(queued.errors.is_empty(), "{:?}", queued.errors);
  let queued = queued.data.into_json().unwrap();
  assert_eq!(queued["subscriberTasksCreateOne"]["status"], "Pending");
  let id = queued["subscriberTasksCreateOne"]["id"].as_str().unwrap();
  let hidden = execute_operation(&schema, &fixture.app, user(SUBSCRIBER_B), Request::new("{ subscriberTasks { nodes { id } } }")).await;
  assert!(hidden.errors.is_empty(), "{:?}", hidden.errors);
  assert!(hidden.data.into_json().unwrap()["subscriberTasks"]["nodes"].as_array().unwrap().is_empty());
  assert!(
    !execute_operation(&schema, &fixture.app, user(SUBSCRIBER_A), Request::new(enqueue(2)))
      .await
      .errors
      .is_empty()
  );
  fixture
    .bootstrap
    .execute_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "UPDATE task_runs SET status = 'Failed' WHERE id = $1",
      [id.into()],
    ))
    .await
    .unwrap();
  let retry = format!(r#"mutation {{ subscriberTasksRetryOne(filter: {{ id: {{ eq: "{id}" }} }}) {{ id status }} }}"#);
  assert!(
    !execute_operation(&schema, &fixture.app, user(SUBSCRIBER_B), Request::new(&retry))
      .await
      .errors
      .is_empty()
  );
  let retried = execute_operation(&schema, &fixture.app, user(SUBSCRIBER_A), Request::new(&retry)).await;
  assert!(retried.errors.is_empty(), "{:?}", retried.errors);
  assert_eq!(retried.data.into_json().unwrap()["subscriberTasksRetryOne"]["status"], "Pending");
  let delete = format!(r#"mutation {{ subscriberTasksDelete(filter: {{ id: {{ eq: "{id}" }} }}) }}"#);
  let denied_delete = execute_operation(&schema, &fixture.app, user(SUBSCRIBER_B), Request::new(&delete)).await;
  assert!(denied_delete.errors.is_empty());
  assert_eq!(denied_delete.data.into_json().unwrap()["subscriberTasksDelete"], 0);
  let deleted = execute_operation(&schema, &fixture.app, user(SUBSCRIBER_A), Request::new(delete)).await;
  assert!(deleted.errors.is_empty());
  assert_eq!(deleted.data.into_json().unwrap()["subscriberTasksDelete"], 1);
  let forbidden_system = execute_operation(&schema, &fixture.app, user(SUBSCRIBER_A), Request::new("{ systemTasks { nodes { id } } }")).await;
  assert!(!forbidden_system.errors.is_empty());
  assert_eq!(forbidden_system.data, Value::Null, "non-null root errors propagate according to the schema");
  fixture
    .bootstrap
    .execute_unprepared("INSERT INTO credential3rd (subscriber_id, credential_type, username) VALUES (1001, 'mikan', 'invalid-fixture-ciphertext')")
    .await
    .unwrap();
  let partial = execute_operation(
    &schema,
    &fixture.app,
    user(SUBSCRIBER_A),
    Request::new("{ subscriptions { nodes { id } } credential3rd { nodes { id broken: username } } }"),
  )
  .await;
  assert!(!partial.errors.is_empty(), "the nullable encrypted field must report a decode error");
  assert_eq!(partial.errors[0].path.last(), Some(&async_graphql::PathSegment::Field("broken".into())));
  let partial = partial.data.into_json().unwrap();
  assert!(
    !partial["subscriptions"]["nodes"].as_array().unwrap().is_empty(),
    "query errors preserve permitted partial data"
  );
  assert!(partial["credential3rd"]["nodes"][0]["id"].is_number());
  assert!(partial["credential3rd"]["nodes"][0]["broken"].is_null());
  let broken_id = partial["credential3rd"]["nodes"][0]["id"].as_i64().unwrap();
  let nullable_mutation_failure = execute_operation(
    &schema,
    &fixture.app,
    user(SUBSCRIBER_A),
    Request::new(format!(
      "mutation {{ credential3rdUpdate(data: {{ userAgent: \"must-rollback\" }}, filter: {{ id: {{ eq: {broken_id} }} }}) {{ id username }} }}"
    )),
  )
  .await;
  assert!(!nullable_mutation_failure.errors.is_empty());
  assert_eq!(nullable_mutation_failure.data, Value::Null);
  let persisted = fixture
    .bootstrap
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      format!("SELECT user_agent FROM credential3rd WHERE id = {broken_id}"),
    ))
    .await
    .unwrap()
    .unwrap();
  assert_eq!(
    persisted.try_get_by_index::<Option<String>>(0).unwrap(),
    None,
    "nullable output errors must roll back successful database writes"
  );
  let cron = r#"mutation { cronCreateOne(data: { cronExpr: "0 0 0 * * *", cronTimezone: "UTC", subscriberTaskCron: { taskType: "sync_one_subscription_sources", subscriptionId: 1, subscriberId: 1002 } }) { id subscriberId subscriberTaskCron } }"#;
  let scheduled = execute_operation(&schema, &fixture.app, user(SUBSCRIBER_A), Request::new(cron)).await;
  assert!(scheduled.errors.is_empty(), "{:?}", scheduled.errors);
  let scheduled = scheduled.data.into_json().unwrap();
  assert_eq!(scheduled["cronCreateOne"]["subscriberId"], SUBSCRIBER_A);
  assert_eq!(scheduled["cronCreateOne"]["subscriberTaskCron"]["subscriberId"], SUBSCRIBER_A);
  fixture
    .bootstrap
    .execute_unprepared(
      r#"
    CREATE FUNCTION reject_fixture_commit() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN
      IF NEW.display_name = 'commit-fails' THEN RAISE EXCEPTION 'fixture deferred constraint'; END IF;
      RETURN NEW;
    END $$;
    CREATE CONSTRAINT TRIGGER fixture_commit_failure AFTER INSERT ON subscriptions
      DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION reject_fixture_commit();
  "#,
    )
    .await
    .unwrap();
  let commit_failure = execute_operation(
    &schema,
    &fixture.app,
    user(SUBSCRIBER_A),
    Request::new(format!("mutation {{ {} }}", create("commit-fails"))),
  )
  .await;
  assert!(!commit_failure.errors.is_empty());
  assert_eq!(commit_failure.data, Value::Null);
  let count = fixture
    .bootstrap
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT count(*) FROM subscriptions WHERE display_name = 'commit-fails'",
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get_by_index::<i64>(0)
    .unwrap();
  assert_eq!(count, 0, "commit failure cannot return success or persist the write");
  let queue_rollback = String::from(
    r#"mutation {{ queued: subscriberTasksCreateOne(data: {{ job: {{ taskType: "sync_one_subscription_sources", subscriptionId: 1 }} }}) {{ id }} invalid: subscriptionsCreateOne(data: {{ displayName: "queue-fails", sourceUrl: "https://example.test/fails", enabled: true, category: mikan_subscriber, credentialId: 2147483647 }}) {{ id }} }}"#,
  );
  let rolled_queue = execute_operation(&schema, &fixture.app, user(SUBSCRIBER_A), Request::new(queue_rollback)).await;
  assert!(!rolled_queue.errors.is_empty());
  assert_eq!(rolled_queue.data, Value::Null);
  let count = fixture
    .bootstrap
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT count(*) FROM task_runs WHERE subscriber_id = 1001",
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get_by_index::<i64>(0)
    .unwrap();
  assert_eq!(count, 1, "a later root error must preserve only the previously cancelled task tombstone");
  let count = fixture
    .bootstrap
    .query_one_raw(Statement::from_string(DbBackend::Postgres, "SELECT count(*) FROM task_outbox"))
    .await
    .unwrap()
    .unwrap()
    .try_get_by_index::<i64>(0)
    .unwrap();
  assert_eq!(count, 2, "the failed root cannot add outbox rows beyond the initial task and manual retry");
  fixture
    .bootstrap
    .execute_unprepared(
      r#"
    CREATE FUNCTION delay_fixture_insert() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN
      IF NEW.display_name = 'cancel-operation' THEN PERFORM pg_sleep(2); END IF;
      RETURN NEW;
    END $$;
    CREATE TRIGGER fixture_cancel BEFORE INSERT ON subscriptions FOR EACH ROW EXECUTE FUNCTION delay_fixture_insert();
  "#,
    )
    .await
    .unwrap();
  let pid = fixture
    .app
    .query_one_raw(Statement::from_string(DbBackend::Postgres, "SELECT pg_backend_pid()"))
    .await
    .unwrap()
    .unwrap()
    .try_get_by_index::<i32>(0)
    .unwrap();
  let cancellable = format!("mutation {{ first: {} second: {} }}", create("cancel-first"), create("cancel-operation"));
  let cancelled_schema = schema.clone();
  let cancelled_db = fixture.app.clone();
  let cancelled = tokio::spawn(async move { execute_operation(&cancelled_schema, &cancelled_db, user(SUBSCRIBER_A), Request::new(cancellable)).await });
  let mut sleeping = false;
  for _ in 0..200 {
    let row = fixture
      .bootstrap
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT wait_event = 'PgSleep' AS sleeping FROM pg_stat_activity WHERE pid = $1",
        [pid.into()],
      ))
      .await
      .unwrap()
      .unwrap();
    if row.try_get_by_index::<Option<bool>>(0).unwrap() == Some(true) {
      sleeping = true;
      break;
    }
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
  }
  assert!(sleeping, "cancellation must happen after the operation reaches its database await");
  cancelled.abort();
  assert!(cancelled.await.unwrap_err().is_cancelled());
  let reused = execute_operation(&schema, &fixture.app, user(SUBSCRIBER_B), Request::new(query)).await;
  assert!(reused.errors.is_empty());
  let count = fixture
    .bootstrap
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT count(*) FROM subscriptions WHERE display_name IN ('cancel-first', 'cancel-operation')",
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get_by_index::<i64>(0)
    .unwrap();
  assert_eq!(count, 0, "cancelled production GraphQL operations roll back before the connection is reused");
  for query in [
    "{ __typename subscriptions { nodes { id } } }",
    "query __schemaAlias { schema: __schema { queryType { name } } ...Business } fragment Business on Query { subscriptions { nodes { id } } }",
  ] {
    let response = schema.execute(Request::new(query).only_introspection()).await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    assert!(
      response.data.into_json().unwrap().get("subscriptions").is_none_or(serde_json::Value::is_null),
      "introspection-only execution must omit business data: {query}"
    );
  }
  assert!(
    schema
      .execute(Request::new("{ __schema { queryType { name } } }").only_introspection())
      .await
      .errors
      .is_empty()
  );
  fixture.close().await;
}
