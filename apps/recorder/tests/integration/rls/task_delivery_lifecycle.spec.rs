use std::{sync::Arc, time::Duration};

use apalis::prelude::Data;
use async_graphql::{Request, dynamic::Schema};
use futures::FutureExt;
use recorder::{
  app::AppContextTrait,
  auth::AuthUserInfo,
  database::operation::IdentityOperation,
  graphql::{build_schema, operation::execute_operation},
  models::auth::{AuthType, Model},
  task::{
    TaskService,
    execution::{CURRENT_FENCE, TaskEnvelope, TaskFence},
  },
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement, TransactionTrait};

use super::{
  support::{RlsFixture, SUBSCRIBER_A, SUBSCRIBER_B},
  task_delivery::{context, context_with_mikan, count, enqueue, state},
};

fn user(owner: i32) -> AuthUserInfo {
  AuthUserInfo {
    auth_type: AuthType::Oidc,
    subscriber_auth: Model {
      id: owner,
      pid: format!("lifecycle-{owner}"),
      issuer: Some("https://fixture.example".into()),
      subscriber_id: owner,
      auth_type: AuthType::Oidc,
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    },
  }
}
async fn gql(schema: &Schema, db: &DatabaseConnection, owner: i32, document: impl Into<String>) -> serde_json::Value {
  let response = execute_operation(schema, db, user(owner), Request::new(document)).await;
  assert!(response.errors.is_empty(), "{:?}", response.errors);
  response.data.into_json().unwrap()
}
async fn subscription_task(fixture: &RlsFixture) -> String {
  let task: recorder::task::SubscriberTask = serde_json::from_value(serde_json::json!({
    "task_type":"sync_one_subscription_sources", "subscriber_id":SUBSCRIBER_A, "subscription_id":1
  }))
  .unwrap();
  let operation = IdentityOperation::begin(&fixture.app, SUBSCRIBER_A).await.unwrap();
  let id = operation
    .run(|db| Box::pin(async move { recorder::task::operation::enqueue(db, "subscriber_task", &task).await }))
    .await
    .unwrap();
  operation.finish(true).await.unwrap();
  id
}
async fn wait_query(db: &DatabaseConnection, predicate: &str, budget: u64) {
  tokio::time::timeout(Duration::from_secs(budget), async {
    loop {
      let present = db
        .query_one_raw(Statement::from_string(
          DbBackend::Postgres,
          format!("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE {predicate})"),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get_by_index::<bool>(0)
        .unwrap();
      if present {
        break;
      }
      tokio::time::sleep(Duration::from_millis(20)).await;
    }
  })
  .await
  .expect("the controlled PostgreSQL phase must be entered before its watchdog");
}
async fn wait_status(fixture: &RlsFixture, id: &str, status: &str) {
  tokio::time::timeout(Duration::from_secs(8), async {
    loop {
      if state(fixture, id).await.0 == status {
        break;
      }
      tokio::time::sleep(Duration::from_millis(20)).await;
    }
  })
  .await
  .expect("durable task status must converge");
}

#[tokio::test]
async fn task_delivery_renewal_polls_locked_business_and_shutdown_releases_fence() {
  let fixture = RlsFixture::new().await;
  let ctx = context(&fixture).await;
  let observer = Database::connect(fixture.bootstrap.config.uri.clone()).await.unwrap();
  let id = subscription_task(&fixture).await;
  let mut lock = Some(fixture.bootstrap.begin().await.unwrap());
  lock
    .as_ref()
    .unwrap()
    .execute_unprepared("LOCK TABLE subscriptions IN ACCESS EXCLUSIVE MODE")
    .await
    .unwrap();
  let app: Arc<dyn AppContextTrait> = ctx.clone();
  let mut execution = tokio::spawn(TaskService::execute(
    TaskEnvelope {
      task_id: id.clone(),
      generation: 1,
    },
    Data::new(app),
  ));
  let mut joined = false;
  let assertions = std::panic::AssertUnwindSafe(async {
    wait_query(&observer, "usename='rls_app' AND wait_event_type='Lock' AND query LIKE '%subscriptions%'", 8).await;
    wait_query(
      &observer,
      "usename='task_control_app' AND wait_event_type='Lock' AND query LIKE '%execution_lease_until%'",
      35,
    )
    .await;
    lock.take().unwrap().rollback().await.unwrap();
    let result = tokio::time::timeout(Duration::from_secs(5), &mut execution)
      .await
      .expect("renewal must not stop polling the now-unblocked execution");
    joined = true;
    let result = result.unwrap();
    assert!(result.is_err(), "the fixture URL is invalid input, not a successful collection");
    assert_eq!(state(&fixture, &id).await.0, "Failed");
    // Exercise shutdown through the actual public worker while a fenced SQL
    // stage is blocked. It must drop the stage before its terminal UPDATE.
    let next = subscription_task(&fixture).await;
    observer
      .execute_unprepared("UPDATE task_outbox SET sent_at=NULL WHERE task_id <> ''")
      .await
      .unwrap();
    lock = Some(fixture.bootstrap.begin().await.unwrap());
    lock
      .as_ref()
      .unwrap()
      .execute_unprepared("LOCK TABLE subscriptions IN ACCESS EXCLUSIVE MODE")
      .await
      .unwrap();
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let coordinate = async {
      wait_query(&observer, "usename='rls_app' AND wait_event_type='Lock' AND query LIKE '%subscriptions%'", 8).await;
      stop.send(()).unwrap();
    };
    let (result, ()) = tokio::time::timeout(Duration::from_secs(12), async {
      tokio::join!(
        ctx.task().run_with_signal(Some(|| async {
          let _ = stopped.await;
        })),
        coordinate
      )
    })
    .await
    .expect("public worker shutdown must join blocked execution and renewal");
    result.unwrap();
    tokio::time::timeout(Duration::from_secs(2), ctx.db().as_ref().execute_unprepared("SELECT 1"))
      .await
      .unwrap()
      .unwrap();
    lock.take().unwrap().rollback().await.unwrap();
    assert_eq!(state(&fixture, &next).await.0, "Scheduled");
    assert_eq!(count(&fixture, "SELECT count(*) FROM task_runs WHERE execution_token IS NOT NULL").await, 0);
  })
  .catch_unwind()
  .await;
  if !joined {
    execution.abort();
    let _ = execution.await;
  }
  if let Some(lock) = lock {
    lock.rollback().await.unwrap();
  }
  observer.close().await.unwrap();
  ctx.task().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
  if let Err(panic) = assertions {
    std::panic::resume_unwind(panic);
  }
}

#[tokio::test]
async fn task_delivery_renewal_sql_failure_drops_the_locked_execution() {
  let fixture = RlsFixture::new().await;
  let ctx = context(&fixture).await;
  let observer = Database::connect(fixture.bootstrap.config.uri.clone()).await.unwrap();
  let id = subscription_task(&fixture).await;
  let mut lock = Some(fixture.bootstrap.begin().await.unwrap());
  lock
    .as_ref()
    .unwrap()
    .execute_unprepared("LOCK TABLE subscriptions IN ACCESS EXCLUSIVE MODE")
    .await
    .unwrap();
  let app: Arc<dyn AppContextTrait> = ctx.clone();
  let mut execution = tokio::spawn(TaskService::execute(
    TaskEnvelope {
      task_id: id.clone(),
      generation: 1,
    },
    Data::new(app),
  ));
  let mut joined = false;
  let assertions = std::panic::AssertUnwindSafe(async {
    wait_query(&observer, "usename='rls_app' AND wait_event_type='Lock' AND query LIKE '%subscriptions%'", 8).await;
    observer
      .execute_unprepared("REVOKE UPDATE ON task_runs FROM konobangu_task_control_access")
      .await
      .unwrap();
    let result = tokio::time::timeout(Duration::from_secs(35), &mut execution)
      .await
      .expect("renewal SQL failure must release the business future");
    joined = true;
    assert!(result.unwrap().is_err());
    tokio::time::timeout(Duration::from_secs(2), ctx.db().as_ref().execute_unprepared("SELECT 1"))
      .await
      .unwrap()
      .unwrap();
    observer
      .execute_unprepared("GRANT UPDATE ON task_runs TO konobangu_task_control_access")
      .await
      .unwrap();
    // The terminal UPDATE also lacked permission; durable recovery keeps the
    // original attempt and can settle after permission repair and lease loss.
    observer
      .execute_unprepared("UPDATE task_runs SET execution_lease_until=clock_timestamp()-interval '1 second',cancel_requested_at=clock_timestamp()")
      .await
      .unwrap();
    assert_eq!(
      recorder::task::operation::settle_cancelled(ctx.task().queue_database().unwrap(), None)
        .await
        .unwrap(),
      1
    );
    lock.take().unwrap().rollback().await.unwrap();
    assert_eq!(state(&fixture, &id).await, ("Killed".into(), 1, 1));
  })
  .catch_unwind()
  .await;
  if !joined {
    execution.abort();
    let _ = execution.await;
  }
  if let Some(lock) = lock {
    lock.rollback().await.unwrap();
  }
  observer.close().await.unwrap();
  ctx.task().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
  if let Err(panic) = assertions {
    std::panic::resume_unwind(panic);
  }
}

#[tokio::test]
async fn task_delivery_cancelled_lost_executors_settle_without_another_envelope() {
  let fixture = RlsFixture::new().await;
  let ctx = context(&fixture).await;
  let assertions = std::panic::AssertUnwindSafe(async {
    let mut ids = Vec::new();
    for (index, attempts) in [1, 1, 3, 1].into_iter().enumerate() {
      let id = enqueue(&fixture).await;
      fixture
        .bootstrap
        .execute_raw(Statement::from_sql_and_values(
          DbBackend::Postgres,
          "UPDATE task_runs SET status='Running',attempts=$2,execution_token=gen_random_uuid(),execution_lease_until=clock_timestamp()+$3*interval '1 second' \
           WHERE id=$1",
          [id.clone().into(), attempts.into(), if index % 2 == 0 { -1_i64 } else { 120 }.into()],
        ))
        .await
        .unwrap();
      let denied = IdentityOperation::begin(&fixture.app, SUBSCRIBER_B).await.unwrap();
      let denied_id = id.clone();
      assert_eq!(
        denied
          .run(|db| Box::pin(async move { recorder::task::operation::cancel_or_archive(db, &denied_id).await }))
          .await
          .unwrap(),
        0
      );
      denied.finish(true).await.unwrap();
      let operation = IdentityOperation::begin(&fixture.app, SUBSCRIBER_A).await.unwrap();
      let cancel_id = id.clone();
      assert_eq!(
        operation
          .run(|db| Box::pin(async move { recorder::task::operation::cancel_or_archive(db, &cancel_id).await }))
          .await
          .unwrap(),
        1
      );
      operation.finish(true).await.unwrap();
      assert_eq!(state(&fixture, &id).await.0, if index % 2 == 0 { "Killed" } else { "Running" });
      ids.push((id, attempts));
    }
    fixture
      .bootstrap
      .execute_unprepared(
        "UPDATE task_outbox SET sent_at=clock_timestamp(); UPDATE task_runs SET execution_lease_until=clock_timestamp()-interval '1 second' WHERE \
         status='Running'",
      )
      .await
      .unwrap();
    assert_eq!(count(&fixture, "SELECT count(*) FROM apalis.jobs").await, 0);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let coordinate = async {
      for (id, attempts) in &ids {
        wait_status(&fixture, id, "Killed").await;
        assert_eq!(state(&fixture, id).await.1, *attempts);
      }
      stop.send(()).unwrap();
    };
    let (result, ()) = tokio::time::timeout(Duration::from_secs(12), async {
      tokio::join!(
        ctx.task().run_with_signal(Some(|| async {
          let _ = stopped.await;
        })),
        coordinate
      )
    })
    .await
    .unwrap();
    result.unwrap();
    assert_eq!(
      count(&fixture, "SELECT count(*) FROM apalis.jobs").await,
      0,
      "recovery does not depend on a new envelope"
    );
    assert_eq!(
      recorder::task::operation::settle_cancelled(ctx.task().queue_database().unwrap(), None)
        .await
        .unwrap(),
      0
    );
    for (id, attempts) in &ids {
      let app: Arc<dyn AppContextTrait> = ctx.clone();
      TaskService::execute(
        TaskEnvelope {
          task_id: id.clone(),
          generation: 1,
        },
        Data::new(app),
      )
      .await
      .unwrap();
      assert_eq!(state(&fixture, id).await, ("Killed".into(), *attempts, 1));
    }
    let id = ids[0].0.clone();
    let operation = IdentityOperation::begin(&fixture.app, SUBSCRIBER_A).await.unwrap();
    let retry_id = id.clone();
    operation
      .run(|db| Box::pin(async move { recorder::task::operation::retry(db, &retry_id).await }))
      .await
      .unwrap();
    operation.finish(true).await.unwrap();
    let app: Arc<dyn AppContextTrait> = ctx.clone();
    TaskService::execute(
      TaskEnvelope {
        task_id: id.clone(),
        generation: 1,
      },
      Data::new(app),
    )
    .await
    .unwrap();
    assert_eq!(state(&fixture, &id).await, ("Pending".into(), 0, 2));
    let stale = TaskFence {
      envelope: TaskEnvelope { task_id: id, generation: 1 },
      token: uuid::Uuid::now_v7(),
    };
    assert!(
      CURRENT_FENCE
        .scope(stale, recorder::database::operation::begin_task_transaction(&fixture.app, SUBSCRIBER_A))
        .await
        .is_err()
    );
    let operation = IdentityOperation::begin(&fixture.app, SUBSCRIBER_A).await.unwrap();
    let archive_id = ids[1].0.clone();
    operation
      .run(|db| Box::pin(async move { recorder::task::operation::cancel_or_archive(db, &archive_id).await }))
      .await
      .unwrap();
    operation.finish(true).await.unwrap();
    assert_eq!(count(&fixture, "SELECT count(*) FROM task_runs WHERE archived_at IS NOT NULL").await, 1);
  })
  .catch_unwind()
  .await;
  ctx.task().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
  if let Err(panic) = assertions {
    std::panic::resume_unwind(panic);
  }
}

#[tokio::test]
async fn task_delivery_subscription_graphql_delete_retains_history_and_rolls_back_cancellation() {
  let fixture = RlsFixture::new().await;
  let ctx = context(&fixture).await;
  let schema = build_schema(ctx.clone(), None, None).unwrap();
  let assertions=std::panic::AssertUnwindSafe(async {
    for (index,(status,archive)) in [("Done",false),("Done",true),("Failed",false),("Failed",true),("Killed",false),("Killed",true),("Pending",false),("Scheduled",false),("Running",false)].into_iter().enumerate() {
      let created=gql(&schema,&fixture.app,SUBSCRIBER_A,format!(r#"mutation {{ subscriptionsCreateOne(data:{{displayName:"history-{index}",sourceUrl:"https://example.test/fixture",enabled:true,category:mikan_subscriber}}) {{id}} }}"#)).await;
      let subscription=created["subscriptionsCreateOne"]["id"].as_i64().unwrap();
      let created=gql(&schema,&fixture.app,SUBSCRIBER_A,format!(r#"mutation {{ subscriberTasksCreateOne(data:{{job:{{taskType:"sync_one_subscription_sources",subscriptionId:{subscription}}}}}) {{id}} }}"#)).await;
      let id=created["subscriberTasksCreateOne"]["id"].as_str().unwrap().to_owned();
      fixture.bootstrap.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,
        "UPDATE task_runs SET status=$2, attempts=CASE WHEN $2='Running' THEN 1 ELSE 0 END,execution_token=CASE WHEN $2='Running' THEN gen_random_uuid() ELSE NULL END,execution_lease_until=CASE WHEN $2='Running' THEN clock_timestamp()+interval '120 seconds' ELSE NULL END WHERE id=$1",
        [id.clone().into(),status.into()])).await.unwrap();
      if archive { gql(&schema,&fixture.app,SUBSCRIBER_A,format!(r#"mutation {{ subscriberTasksDelete(filter:{{id:{{eq:"{id}"}}}}) }}"#)).await; }
      let denied=gql(&schema,&fixture.app,SUBSCRIBER_B,format!("mutation {{ subscriptionsDelete(filter:{{id:{{eq:{subscription}}}}}) }}")).await;
      assert_eq!(denied["subscriptionsDelete"],0);
      let deleted=gql(&schema,&fixture.app,SUBSCRIBER_A,format!("mutation {{ subscriptionsDelete(filter:{{id:{{eq:{subscription}}}}}) }}")).await;
      assert_eq!(deleted["subscriptionsDelete"],1);
      let row=fixture.bootstrap.query_one_raw(Statement::from_sql_and_values(DbBackend::Postgres,
        "SELECT subscription_id,payload->>'subscription_id' AS original FROM task_runs WHERE id=$1",[id.clone().into()])).await.unwrap().unwrap();
      assert_eq!(row.try_get::<Option<i32>>("","subscription_id").unwrap(),None);
      assert_eq!(row.try_get::<String>("","original").unwrap(),subscription.to_string());
      if !archive {
        let detail=gql(&schema,&fixture.app,SUBSCRIBER_A,format!(r#"{{ subscriberTasks(filter:{{id:{{eq:"{id}"}}}}) {{nodes {{id subscription {{id}}}}}} }}"#)).await;
        assert!(detail["subscriberTasks"]["nodes"][0]["subscription"].is_null());
      }
      fixture.bootstrap.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,
        "UPDATE task_runs SET execution_lease_until=clock_timestamp()-interval '1 second' WHERE id=$1 AND status='Running'",[id.clone().into()])).await.unwrap();
      let app:Arc<dyn AppContextTrait>=ctx.clone();
      TaskService::execute(TaskEnvelope{task_id:id.clone(),generation:1},Data::new(app)).await.unwrap();
      let expected=if matches!(status,"Pending"|"Scheduled"|"Running") {"Killed"} else {status};
      assert_eq!(state(&fixture,&id).await.0,expected);
      let retry=execute_operation(&schema,&fixture.app,user(SUBSCRIBER_A),Request::new(format!(r#"mutation {{subscriberTasksRetryOne(filter:{{id:{{eq:"{id}"}}}})}}"#))).await;
      assert!(!retry.errors.is_empty(),"deleted-resource retries must fail explicitly");
    }
    let created=gql(&schema,&fixture.app,SUBSCRIBER_A,r#"mutation {subscriberTasksCreateOne(data:{job:{taskType:"sync_one_subscription_sources",subscriptionId:1}}){id}}"#).await;
    let id=created["subscriberTasksCreateOne"]["id"].as_str().unwrap();
    for later in [
      r#"subscriberTasksCreateOne(data:{job:{taskType:"sync_one_subscription_sources",subscriptionId:2147483647}}){id}"#,
      r#"subscriptionsCreateBatch(data:[{displayName:"rollback",sourceUrl:"https://example.test/a",category:mikan_subscriber,enabled:true},{displayName:"bad",sourceUrl:"https://example.test/b",category:mikan_subscriber,enabled:true,credentialId:2147483647}]){id}"#,
    ] {
      let failed=execute_operation(&schema,&fixture.app,user(SUBSCRIBER_A),Request::new(format!("mutation {{ subscriptionsDelete(filter:{{id:{{eq:1}}}}) {later} }}"))).await;
      assert!(!failed.errors.is_empty());
      assert_eq!(count(&fixture,"SELECT count(*) FROM subscriptions WHERE id=1").await,1);
      assert_eq!(state(&fixture,id).await,("Pending".into(),0,1));
      assert_eq!(count(&fixture,"SELECT count(*) FROM task_runs WHERE subscription_id=1 AND cancel_requested_at IS NOT NULL").await,0);
    }
    assert_eq!(count(&fixture,"SELECT count(*) FROM task_runs").await,10);
    assert_eq!(count(&fixture,"SELECT count(*) FROM task_outbox").await,10);
  }).catch_unwind().await;
  ctx.task().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
  if let Err(panic) = assertions {
    std::panic::resume_unwind(panic);
  }
}

#[tokio::test]
async fn task_delivery_subscription_delete_waits_for_fenced_stage_then_rejects_late_results() {
  let fixture = RlsFixture::new().await;
  let ctx = context(&fixture).await;
  let schema = build_schema(ctx.clone(), None, None).unwrap();
  let observer = Database::connect(fixture.bootstrap.config.uri.clone()).await.unwrap();
  let concurrent = Database::connect(fixture.app_uri.clone()).await.unwrap();
  let id = subscription_task(&fixture).await;
  let token = uuid::Uuid::now_v7();
  fixture
    .bootstrap
    .execute_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "UPDATE task_runs SET status='Running',attempts=1,execution_token=$2,execution_lease_until=clock_timestamp()+interval '120 seconds' WHERE id=$1",
      [id.clone().into(), token.into()],
    ))
    .await
    .unwrap();
  let fence = TaskFence {
    envelope: TaskEnvelope {
      task_id: id.clone(),
      generation: 1,
    },
    token,
  };
  let mut stage = Some(
    CURRENT_FENCE
      .scope(
        fence.clone(),
        recorder::database::operation::begin_task_transaction(ctx.db().as_ref(), SUBSCRIBER_A),
      )
      .await
      .unwrap(),
  );
  let schema_for_delete = schema.clone();
  let delete_pool = fixture.app.clone();
  let mut deletion = tokio::spawn(async move {
    execute_operation(
      &schema_for_delete,
      &delete_pool,
      user(SUBSCRIBER_A),
      Request::new("mutation { subscriptionsDelete(filter:{id:{eq:1}}) }"),
    )
    .await
  });
  let mut joined = false;
  let assertions = std::panic::AssertUnwindSafe(async {
    wait_query(&observer, "usename='rls_app' AND wait_event_type='Lock' AND query LIKE '%task_runs%'", 8).await;
    let raced = execute_operation(
      &schema,
      &concurrent,
      user(SUBSCRIBER_A),
      Request::new(r#"mutation {subscriberTasksCreateOne(data:{job:{taskType:"sync_one_subscription_sources",subscriptionId:1}}){id}}"#),
    )
    .await;
    assert!(!raced.errors.is_empty(), "creation cannot pass a deletion intent");
    stage
      .as_ref()
      .unwrap()
      .execute_unprepared("UPDATE subscriptions SET display_name='valid preceding stage' WHERE id=1")
      .await
      .unwrap();
    CURRENT_FENCE
      .scope(fence.clone(), recorder::database::operation::commit_task_transaction(stage.take().unwrap()))
      .await
      .unwrap();
    let result = tokio::time::timeout(Duration::from_secs(5), &mut deletion).await.unwrap();
    joined = true;
    let deleted = result.unwrap();
    assert!(deleted.errors.is_empty(), "{:?}", deleted.errors);
    assert_eq!(deleted.data.into_json().unwrap()["subscriptionsDelete"], 1);
    assert!(
      CURRENT_FENCE
        .scope(fence, recorder::database::operation::begin_task_transaction(ctx.db().as_ref(), SUBSCRIBER_A))
        .await
        .is_err()
    );
    fixture
      .bootstrap
      .execute_unprepared("UPDATE task_runs SET execution_lease_until=clock_timestamp()-interval '1 second' WHERE status='Running'")
      .await
      .unwrap();
    let app: Arc<dyn AppContextTrait> = ctx.clone();
    TaskService::execute(
      TaskEnvelope {
        task_id: id.clone(),
        generation: 1,
      },
      Data::new(app),
    )
    .await
    .unwrap();
    assert_eq!(state(&fixture, &id).await, ("Killed".into(), 1, 1));
    assert_eq!(count(&fixture, "SELECT count(*) FROM task_runs").await, 1);
    assert_eq!(count(&fixture, "SELECT count(*) FROM task_outbox").await, 1);
  })
  .catch_unwind()
  .await;
  if !joined {
    deletion.abort();
    let _ = deletion.await;
  }
  if let Some(stage) = stage {
    stage.rollback().await.unwrap();
  }
  concurrent.close().await.unwrap();
  observer.close().await.unwrap();
  ctx.task().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
  if let Err(panic) = assertions {
    std::panic::resume_unwind(panic);
  }
}

#[tokio::test]
async fn task_delivery_subscription_delete_detaches_archived_history() {
  let fixture = RlsFixture::new().await;
  let ctx = context(&fixture).await;
  let schema = build_schema(ctx.clone(), None, None).unwrap();
  let id = subscription_task(&fixture).await;
  let assertions = std::panic::AssertUnwindSafe(async {
    fixture
      .bootstrap
      .execute_unprepared("UPDATE task_runs SET status='Done',archived_at=clock_timestamp()")
      .await
      .unwrap();
    let fixed = fixture
      .bootstrap
      .query_one_raw(Statement::from_string(
        DbBackend::Postgres,
        "SELECT confdeltype::text AS action FROM pg_constraint WHERE conrelid='task_runs'::regclass AND conname='task_runs_subscription_id_fkey'",
      ))
      .await
      .unwrap()
      .unwrap();
    assert_eq!(fixed.try_get::<String>("", "action").unwrap(), "n");
    let deleted = gql(&schema, &fixture.app, SUBSCRIBER_A, "mutation {subscriptionsDelete(filter:{id:{eq:1}})}").await;
    assert_eq!(deleted["subscriptionsDelete"], 1);
    assert_eq!(state(&fixture, &id).await, ("Done".into(), 0, 1));
    assert_eq!(
      count(
        &fixture,
        "SELECT count(*) FROM task_runs WHERE subscription_id IS NULL AND payload->>'subscription_id'='1'"
      )
      .await,
      1
    );
    assert_eq!(count(&fixture, "SELECT count(*) FROM task_outbox").await, 1);
  })
  .catch_unwind()
  .await;
  ctx.task().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
  if let Err(panic) = assertions {
    std::panic::resume_unwind(panic);
  }
}

#[tokio::test]
async fn task_delivery_renewal_cancellation_and_lease_loss_stop_owned_http_futures() {
  let fixture = RlsFixture::new().await;
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let address = listener.local_addr().unwrap();
  let (entered, mut entries) = tokio::sync::mpsc::unbounded_channel();
  let (release, released) = tokio::sync::watch::channel(false);
  let router = axum::Router::new().fallback(axum::routing::get(move || {
    let entered = entered.clone();
    let mut released = released.clone();
    async move {
      let _ = entered.send(());
      let _ = released.wait_for(|value| *value).await;
      http::StatusCode::SERVICE_UNAVAILABLE
    }
  }));
  let (http_stop, http_stopped) = tokio::sync::oneshot::channel();
  let mut server = tokio::spawn(async move {
    axum::serve(listener, router)
      .with_graceful_shutdown(async {
        let _ = http_stopped.await;
      })
      .await
      .unwrap();
  });
  let mikan = recorder::extract::mikan::MikanClient::from_config(recorder::extract::mikan::MikanConfig {
    base_url: format!("http://{address}/").parse().unwrap(),
    http_client: fetch::HttpClientConfig {
      exponential_backoff_max_retries: Some(0),
      ..Default::default()
    },
  })
  .await
  .unwrap();
  let ctx = context_with_mikan(&fixture, Some(mikan)).await;
  fixture
    .bootstrap
    .execute_unprepared("UPDATE subscriptions SET source_url='https://example.test/RSS/MyBangumi?token=fixture' WHERE id=1")
    .await
    .unwrap();
  let mut ids = Vec::new();
  let mut executions = Vec::new();
  for _ in 0..2 {
    let task: recorder::task::SubscriberTask =
      serde_json::from_value(serde_json::json!({"task_type":"sync_one_subscription_feeds_incremental","subscriber_id":SUBSCRIBER_A,"subscription_id":1}))
        .unwrap();
    let operation = IdentityOperation::begin(&fixture.app, SUBSCRIBER_A).await.unwrap();
    let id = operation
      .run(|db| Box::pin(async move { recorder::task::operation::enqueue(db, "subscriber_task", &task).await }))
      .await
      .unwrap();
    operation.finish(true).await.unwrap();
    let app: Arc<dyn AppContextTrait> = ctx.clone();
    executions.push((
      tokio::spawn(TaskService::execute(
        TaskEnvelope {
          task_id: id.clone(),
          generation: 1,
        },
        Data::new(app),
      )),
      false,
    ));
    ids.push(id);
  }
  let assertions = std::panic::AssertUnwindSafe(async {
    for _ in 0..2 {
      tokio::time::timeout(Duration::from_secs(8), entries.recv()).await.unwrap().unwrap();
    }
    let operation = IdentityOperation::begin(&fixture.app, SUBSCRIBER_A).await.unwrap();
    let id = ids[0].clone();
    operation
      .run(|db| Box::pin(async move { recorder::task::operation::cancel_or_archive(db, &id).await }))
      .await
      .unwrap();
    operation.finish(true).await.unwrap();
    fixture
      .bootstrap
      .execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE task_runs SET execution_lease_until=clock_timestamp()-interval '1 second' WHERE id=$1",
        [ids[1].clone().into()],
      ))
      .await
      .unwrap();
    for (execution, joined) in &mut executions {
      let result = tokio::time::timeout(Duration::from_secs(35), execution)
        .await
        .expect("cancelled/lost renewal must stop the owned business future");
      *joined = true;
      assert!(result.unwrap().is_err());
    }
    assert_eq!(state(&fixture, &ids[0]).await, ("Killed".into(), 1, 1));
    assert_eq!(state(&fixture, &ids[1]).await, ("Running".into(), 1, 1));
    let operation = IdentityOperation::begin(&fixture.app, SUBSCRIBER_A).await.unwrap();
    let id = ids[1].clone();
    operation
      .run(|db| Box::pin(async move { recorder::task::operation::cancel_or_archive(db, &id).await }))
      .await
      .unwrap();
    operation.finish(true).await.unwrap();
    assert_eq!(state(&fixture, &ids[1]).await, ("Killed".into(), 1, 1));
    assert_eq!(count(&fixture, "SELECT count(*) FROM task_runs WHERE execution_token IS NOT NULL").await, 0);
    assert_eq!(
      count(
        &fixture,
        "SELECT count(*) FROM pg_stat_activity WHERE usename='rls_app' AND state='idle in transaction'"
      )
      .await,
      0
    );
  })
  .catch_unwind()
  .await;
  for (execution, joined) in executions {
    if !joined {
      execution.abort();
      let _ = execution.await;
    }
  }
  release.send_replace(true);
  let _ = http_stop.send(());
  if tokio::time::timeout(Duration::from_secs(5), &mut server).await.is_err() {
    server.abort();
    let _ = server.await;
  }
  ctx.task().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
  if let Err(panic) = assertions {
    std::panic::resume_unwind(panic);
  }
}
