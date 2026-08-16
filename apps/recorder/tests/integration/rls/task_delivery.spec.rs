use std::{sync::Arc, time::Duration};

use apalis::prelude::*;
use chrono::{TimeZone, Utc};
use recorder::{
  app::AppContextTrait,
  database::{DatabaseService, operation::IdentityOperation},
  task::{
    TaskConfig, TaskService, delivery,
    execution::{CURRENT_FENCE, TaskEnvelope, TaskFence},
  },
  test_utils::app::TestingAppContext,
};
use sea_orm::{ConnectionTrait, DbBackend, Statement};

use super::support::{RlsFixture, SUBSCRIBER_A};

pub(super) async fn context(fixture: &RlsFixture) -> Arc<TestingAppContext> {
  context_with_mikan(fixture, None).await
}
pub(super) async fn context_with_mikan(fixture: &RlsFixture, mikan: Option<recorder::extract::mikan::MikanClient>) -> Arc<TestingAppContext> {
  let mut config = fixture.bootstrap.config.clone();
  config.uri = fixture.app_uri.clone();
  config.auto_migrate = false;
  let context = Arc::new(
    TestingAppContext::builder()
      .db(DatabaseService::from_config(config).await.unwrap())
      .mikan(mikan.unwrap_or(recorder::test_utils::mikan::build_testing_mikan_client("http://127.0.0.1:1/").await.unwrap()))
      .build(),
  );
  context.set_task(
    TaskService::from_config_and_ctx(
      TaskConfig {
        queue_database_uri: Some(fixture.queue_uri.clone()),
        cron_interval_duration: Duration::from_millis(50),
        ..Default::default()
      },
      context.clone(),
    )
    .await
    .unwrap(),
  );
  context
}
pub(super) async fn enqueue(fixture: &RlsFixture) -> String {
  let operation = IdentityOperation::begin(&fixture.app, SUBSCRIBER_A).await.unwrap();
  let task: recorder::task::SystemTask = recorder::task::EchoTask::builder()
    .task_id("delivery-fixture".into())
    .subscriber_id(Some(SUBSCRIBER_A))
    .build()
    .into();
  let id = operation
    .run(|db| Box::pin(async move { recorder::task::operation::enqueue(db, "system_task", &task).await }))
    .await
    .unwrap();
  operation.finish(true).await.unwrap();
  id
}
pub(super) async fn state(fixture: &RlsFixture, id: &str) -> (String, i32, i64) {
  let row = fixture
    .bootstrap
    .query_one_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "SELECT status,attempts,generation FROM task_runs WHERE id=$1",
      [id.into()],
    ))
    .await
    .unwrap()
    .unwrap();
  (
    row.try_get("", "status").unwrap(),
    row.try_get("", "attempts").unwrap(),
    row.try_get("", "generation").unwrap(),
  )
}
pub(super) async fn count(fixture: &RlsFixture, sql: &str) -> i64 {
  fixture
    .bootstrap
    .query_one_raw(Statement::from_string(DbBackend::Postgres, sql))
    .await
    .unwrap()
    .unwrap()
    .try_get_by_index(0)
    .unwrap()
}

#[tokio::test]
async fn task_delivery_real_worker_duplicate_outbox_crash_and_terminal_replay() {
  let fixture = RlsFixture::new().await;
  let ctx = context(&fixture).await;
  let id = enqueue(&fixture).await;
  let queue = ctx.task().queue_database().unwrap();
  let (a, b) = tokio::join!(delivery::dispatch_one(queue), delivery::dispatch_one(queue));
  assert_eq!(usize::from(a.unwrap()) + usize::from(b.unwrap()), 1);
  assert_eq!(count(&fixture, "SELECT count(*) FROM apalis.jobs").await, 1);
  // Simulate a crash after durable push and before the application's sent mark.
  fixture
    .bootstrap
    .execute_unprepared("UPDATE task_outbox SET sent_at=NULL, lease_token=NULL, lease_until=NULL")
    .await
    .unwrap();
  assert!(delivery::dispatch_one(queue).await.unwrap());
  assert_eq!(count(&fixture, "SELECT count(*) FROM apalis.jobs").await, 1);
  assert_eq!(count(&fixture, "SELECT count(*) FROM task_outbox WHERE sent_at IS NOT NULL").await, 1);
  delivery::storage(queue)
    .push(TaskEnvelope {
      task_id: id.clone(),
      generation: 1,
    })
    .await
    .unwrap();
  let (stop, stopped) = tokio::sync::oneshot::channel();
  let worker = ctx.task().run_with_signal(Some(|| async {
    let _ = stopped.await;
  }));
  let assertion = async {
    tokio::time::timeout(Duration::from_secs(10), async {
      loop {
        if state(&fixture, &id).await.0 == "Done" {
          break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
      }
    })
    .await
    .unwrap();
    assert_eq!(state(&fixture, &id).await, ("Done".into(), 1, 1));
    // Terminal and old generation envelopes cannot start new business work.
    delivery::storage(queue)
      .push(TaskEnvelope {
        task_id: id.clone(),
        generation: 1,
      })
      .await
      .unwrap();
    delivery::storage(queue)
      .push(TaskEnvelope {
        task_id: id.clone(),
        generation: 0,
      })
      .await
      .unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
      loop {
        if count(&fixture, "SELECT count(*) FROM apalis.jobs WHERE status IN ('Pending','Running','Queued')").await == 0 {
          break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
      }
    })
    .await
    .unwrap();
    assert_eq!(state(&fixture, &id).await.1, 1);
    let _ = stop.send(());
  };
  let (run, ()) = tokio::join!(worker, assertion);
  run.unwrap();
  let takeover = enqueue(&fixture).await;
  fixture
    .bootstrap
    .execute_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "UPDATE task_runs SET status='Running',attempts=1,execution_token=gen_random_uuid(),execution_lease_until=clock_timestamp()-interval '1 second' WHERE \
       id=$1",
      [takeover.clone().into()],
    ))
    .await
    .unwrap();
  let (stop, stopped) = tokio::sync::oneshot::channel();
  let assertion = async {
    tokio::time::timeout(Duration::from_secs(10), async {
      while state(&fixture, &takeover).await.0 != "Done" {
        tokio::time::sleep(Duration::from_millis(20)).await;
      }
    })
    .await
    .unwrap();
    let _ = stop.send(());
  };
  let (result, ()) = tokio::join!(
    ctx.task().run_with_signal(Some(|| async {
      let _ = stopped.await;
    })),
    assertion
  );
  result.unwrap();
  assert_eq!(state(&fixture, &takeover).await.1, 2);
  // Clearing queue history still cannot resurrect a completed business run.
  fixture
    .bootstrap
    .execute_unprepared("DELETE FROM apalis.jobs; UPDATE task_outbox SET sent_at=NULL,lease_token=NULL,lease_until=NULL")
    .await
    .unwrap();
  while delivery::dispatch_one(queue).await.unwrap() {}
  assert_eq!(count(&fixture, "SELECT count(*) FROM apalis.jobs").await, 0);
  assert_eq!(state(&fixture, &id).await.1, 1);
  queue.clone().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
}

#[tokio::test]
async fn task_delivery_cancel_retry_competition_and_stale_business_fences() {
  let fixture = RlsFixture::new().await;
  let ctx = context(&fixture).await;
  let id = enqueue(&fixture).await;
  let operation = IdentityOperation::begin(&fixture.app, SUBSCRIBER_A).await.unwrap();
  let cancel_id = id.clone();
  operation
    .run(|db| Box::pin(async move { recorder::task::operation::cancel_or_archive(db, &cancel_id).await }))
    .await
    .unwrap();
  operation.finish(true).await.unwrap();
  assert_eq!(state(&fixture, &id).await.0, "Killed");
  let retry = |connection: sea_orm::DatabaseConnection| {
    let id = id.clone();
    async move {
      let operation = IdentityOperation::begin(&connection, SUBSCRIBER_A).await.unwrap();
      let result = operation
        .run(|db| Box::pin(async move { recorder::task::operation::retry(db, &id).await }))
        .await;
      operation.finish(result.is_ok()).await.unwrap();
      result
    }
  };
  let (a, b) = tokio::join!(retry(fixture.app.clone()), retry(ctx.db().as_ref().clone()));
  assert_eq!(usize::from(a.is_ok()) + usize::from(b.is_ok()), 1);
  assert_eq!(state(&fixture, &id).await, ("Pending".into(), 0, 2));
  assert_eq!(count(&fixture, "SELECT count(*) FROM task_outbox").await, 2);
  let old = uuid::Uuid::now_v7();
  let new = uuid::Uuid::now_v7();
  fixture
    .bootstrap
    .execute_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "UPDATE task_runs SET status='Running', execution_token=$2, execution_lease_until=clock_timestamp()+INTERVAL '120 seconds' WHERE id=$1",
      [id.clone().into(), new.into()],
    ))
    .await
    .unwrap();
  let stale = TaskFence {
    envelope: TaskEnvelope {
      task_id: id.clone(),
      generation: 2,
    },
    token: old,
  };
  assert!(
    CURRENT_FENCE
      .scope(stale, recorder::database::operation::begin_task_transaction(&fixture.app, SUBSCRIBER_A))
      .await
      .is_err()
  );
  let storage_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("../../temp/verification/fixtures/storage-fence")
    .join(uuid::Uuid::now_v7().to_string());
  let storage = recorder::storage::StorageService::from_config(recorder::storage::StorageConfig {
    data_dir: storage_root.to_string_lossy().into_owned(),
  })
  .await
  .unwrap();
  storage.write("fixture.webp", bytes::Bytes::from_static(b"current")).await.unwrap();
  let stale = TaskFence {
    envelope: TaskEnvelope {
      task_id: id.clone(),
      generation: 2,
    },
    token: old,
  };
  assert!(
    CURRENT_FENCE
      .scope(
        stale,
        storage.write_for_task(ctx.task().queue_database().unwrap(), "fixture.webp", bytes::Bytes::from_static(b"stale"))
      )
      .await
      .is_err()
  );
  assert_eq!(storage.read("fixture.webp").await.unwrap().to_bytes().as_ref(), b"current");
  let current = TaskFence {
    envelope: TaskEnvelope {
      task_id: id.clone(),
      generation: 2,
    },
    token: new,
  };
  CURRENT_FENCE
    .scope(current, async {
      let transaction = recorder::database::operation::begin_task_transaction(&fixture.app, SUBSCRIBER_A).await.unwrap();
      transaction
        .execute_unprepared("UPDATE subscriptions SET display_name='must-roll-back' WHERE id=1")
        .await
        .unwrap();
      transaction
        .execute_unprepared("UPDATE task_runs SET execution_lease_until=clock_timestamp()-INTERVAL '1 second'")
        .await
        .unwrap();
      assert!(recorder::database::operation::commit_task_transaction(transaction).await.is_err());
    })
    .await;
  assert_eq!(
    count(&fixture, "SELECT count(*) FROM subscriptions WHERE display_name='must-roll-back'").await,
    0
  );
  assert!(
    ctx
      .task()
      .queue_database()
      .unwrap()
      .execute_unprepared("SELECT * FROM subscriptions")
      .await
      .is_err()
  );
  assert!(
    ctx
      .task()
      .queue_database()
      .unwrap()
      .execute_unprepared("SELECT * FROM auth_identity.auth")
      .await
      .is_err()
  );
  assert!(fixture.app.execute_unprepared("SELECT * FROM apalis.jobs").await.is_err());
  ctx.task().queue_database().unwrap().clone().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
}

#[tokio::test]
async fn task_delivery_cron_occurrence_atomicity_coalescing_and_no_overlap() {
  let fixture = RlsFixture::new().await;
  let ctx = context(&fixture).await;
  let now = Utc.with_ymd_and_hms(2030, 1, 2, 12, 0, 0).unwrap();
  fixture
    .bootstrap
    .execute_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "INSERT INTO cron (cron_expr,cron_timezone,next_run,enabled,system_task_cron) VALUES ('0 * * * * \
       *','UTC',$1,true,'{\"task_type\":\"test\",\"task_id\":\"cron-fixture\",\"subscriber_id\":null,\"cron_id\":null}')",
      [(now - chrono::Duration::hours(24)).into()],
    ))
    .await
    .unwrap();
  let queue = ctx.task().queue_database().unwrap();
  let (a, b) = tokio::join!(
    recorder::models::cron::Model::dispatch_due(queue, now),
    recorder::models::cron::Model::dispatch_due(queue, now)
  );
  assert_eq!(a.unwrap() + b.unwrap(), 1);
  assert_eq!(count(&fixture, "SELECT count(*) FROM task_runs WHERE cron_id IS NOT NULL").await, 1);
  assert_eq!(count(&fixture, "SELECT count(*) FROM task_outbox").await, 1);
  assert_eq!(
    recorder::models::cron::Model::dispatch_due(queue, now + chrono::Duration::minutes(1))
      .await
      .unwrap(),
    1
  );
  assert_eq!(count(&fixture, "SELECT count(*) FROM task_runs WHERE cron_id IS NOT NULL").await, 1);
  assert_eq!(
    count(&fixture, "SELECT count(*) FROM cron WHERE last_error LIKE 'Occurrence skipped:%'").await,
    1
  );
  fixture.bootstrap.execute_unprepared("UPDATE cron SET enabled=false").await.unwrap();
  assert_eq!(
    recorder::models::cron::Model::dispatch_due(queue, now + chrono::Duration::hours(1))
      .await
      .unwrap(),
    0
  );
  queue.clone().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
}

#[tokio::test]
async fn task_delivery_real_worker_retries_5_30_budget_permanent_4xx_and_restart() {
  use std::sync::atomic::{AtomicUsize, Ordering};

  use futures::FutureExt;
  let fixture = RlsFixture::new().await;
  let status = Arc::new(AtomicUsize::new(503));
  let observed = Arc::new(std::sync::Mutex::new(Vec::new()));
  let handler_status = status.clone();
  let handler_observed = observed.clone();
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let address = listener.local_addr().unwrap();
  let router = axum::Router::new().fallback(axum::routing::get(move || {
    let status = handler_status.clone();
    let observed = handler_observed.clone();
    async move {
      observed.lock().unwrap().push(tokio::time::Instant::now());
      http::StatusCode::from_u16(status.load(Ordering::SeqCst) as u16).unwrap()
    }
  }));
  let (http_stop, http_stopped) = tokio::sync::oneshot::channel();
  let server = tokio::spawn(async move {
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
  let task: recorder::task::SubscriberTask = recorder::task::SyncOneSubscriptionFeedsIncrementalTask::builder()
    .subscription_id(1)
    .subscriber_id(SUBSCRIBER_A)
    .build()
    .into();
  let operation = IdentityOperation::begin(&fixture.app, SUBSCRIBER_A).await.unwrap();
  let id = operation
    .run(|db| Box::pin(async move { recorder::task::operation::enqueue(db, "subscriber_task", &task).await }))
    .await
    .unwrap();
  operation.finish(true).await.unwrap();
  // A dropped outbox claimant is retried after its persisted lease expires.
  fixture
    .bootstrap
    .execute_unprepared("UPDATE task_outbox SET lease_token=gen_random_uuid(),lease_until=clock_timestamp()-interval '1 second'")
    .await
    .unwrap();
  let assertions = std::panic::AssertUnwindSafe(async {
    async fn until(fixture: &RlsFixture, id: &str, wanted: &str) {
      let waited = tokio::time::timeout(Duration::from_secs(50), async {
        loop {
          if state(fixture, id).await.0 == wanted {
            break;
          }
          tokio::time::sleep(Duration::from_millis(20)).await;
        }
      })
      .await;
      if waited.is_err() {
        let jobs = fixture
          .bootstrap
          .query_one_raw(Statement::from_string(
            DbBackend::Postgres,
            "SELECT json_agg(row_to_json(j)) AS jobs FROM apalis.jobs j",
          ))
          .await
          .unwrap()
          .unwrap();
        panic!(
          "Waiting for {wanted}: business={:?}, queue={:?}",
          state(fixture, id).await,
          jobs.try_get::<Option<serde_json::Value>>("", "jobs").unwrap()
        );
      }
    }
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let assertion = async {
      until(&fixture, &id, "Scheduled").await;
      let _ = stop.send(());
    };
    let (result, ()) = tokio::join!(
      ctx.task().run_with_signal(Some(|| async {
        let _ = stopped.await;
      })),
      assertion
    );
    result.unwrap();
    assert_eq!(state(&fixture, &id).await.1, 1);
    // Restart a real public worker. Failed queue delivery is reclaimed through
    // Apalis while business run_at still enforces the committed retry delay.
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let assertion = async {
      until(&fixture, &id, "Failed").await;
      let _ = stop.send(());
    };
    let (result, ()) = tokio::join!(
      ctx.task().run_with_signal(Some(|| async {
        let _ = stopped.await;
      })),
      assertion
    );
    result.unwrap();
    assert_eq!(state(&fixture, &id).await, ("Failed".into(), 3, 1));
    let calls = observed.lock().unwrap().clone();
    assert_eq!(calls.len(), 3);
    assert!(calls[1].duration_since(calls[0]) >= Duration::from_secs(5));
    assert!(calls[2].duration_since(calls[1]) >= Duration::from_secs(30));
    status.store(404, Ordering::SeqCst);
    observed.lock().unwrap().clear();
    let operation = IdentityOperation::begin(&fixture.app, SUBSCRIBER_A).await.unwrap();
    let retry_id = id.clone();
    operation
      .run(|db| Box::pin(async move { recorder::task::operation::retry(db, &retry_id).await }))
      .await
      .unwrap();
    operation.finish(true).await.unwrap();
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let assertion = async {
      until(&fixture, &id, "Failed").await;
      let _ = stop.send(());
    };
    let (result, ()) = tokio::join!(
      ctx.task().run_with_signal(Some(|| async {
        let _ = stopped.await;
      })),
      assertion
    );
    result.unwrap();
    assert_eq!(state(&fixture, &id).await, ("Failed".into(), 1, 2));
    assert_eq!(observed.lock().unwrap().len(), 1);
    assert!(serde_json::from_value::<TaskEnvelope>(serde_json::json!({"task_id":id,"generation":2,"subscriber_id":999})).is_err());
  })
  .catch_unwind()
  .await;
  let _ = http_stop.send(());
  server.await.unwrap();
  ctx.task().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
  if let Err(panic) = assertions {
    std::panic::resume_unwind(panic);
  }
}

#[tokio::test]
async fn task_delivery_outbox_failure_backoff_lease_and_cron_transaction_rollback() {
  let fixture = RlsFixture::new().await;
  let ctx = context(&fixture).await;
  let id = enqueue(&fixture).await;
  let queue = ctx.task().queue_database().unwrap();
  fixture
    .bootstrap
    .execute_unprepared("REVOKE INSERT ON apalis.jobs FROM konobangu_task_control_access")
    .await
    .unwrap();
  for seconds in [1_i64, 2, 4, 8, 16, 32, 60, 60] {
    fixture
      .bootstrap
      .execute_unprepared("UPDATE task_outbox SET available_at=clock_timestamp()-interval '1 second'")
      .await
      .unwrap();
    assert!(delivery::dispatch_one(queue).await.unwrap());
    let row = fixture
      .bootstrap
      .query_one_raw(Statement::from_string(
        DbBackend::Postgres,
        "SELECT sent_at IS NULL AND lease_token IS NULL AND last_error='Queue delivery failed' AS retained,extract(epoch FROM \
         available_at-clock_timestamp())::double precision AS delay FROM task_outbox",
      ))
      .await
      .unwrap()
      .unwrap();
    assert!(row.try_get::<bool>("", "retained").unwrap());
    let delay = row.try_get::<f64>("", "delay").unwrap();
    assert!(delay <= seconds as f64 && delay > seconds as f64 - 1.0);
  }
  fixture
    .bootstrap
    .execute_unprepared(
      "GRANT INSERT ON apalis.jobs TO konobangu_task_control_access; UPDATE task_outbox SET available_at=clock_timestamp()-interval '1 \
       second',lease_token=gen_random_uuid(),lease_until=clock_timestamp()+interval '60 seconds'",
    )
    .await
    .unwrap();
  assert!(!delivery::dispatch_one(queue).await.unwrap());
  fixture
    .bootstrap
    .execute_unprepared("UPDATE task_outbox SET lease_until=clock_timestamp()-interval '1 second'")
    .await
    .unwrap();
  assert!(delivery::dispatch_one(queue).await.unwrap());
  assert_eq!(count(&fixture, "SELECT count(*) FROM apalis.jobs").await, 1);
  let now = Utc.with_ymd_and_hms(2030, 1, 2, 12, 0, 0).unwrap();
  fixture
    .bootstrap
    .execute_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "INSERT INTO cron (cron_expr,cron_timezone,next_run,enabled,system_task_cron) VALUES ('0 * * * * \
       *','UTC',$1,true,'{\"task_type\":\"test\",\"task_id\":\"rollback\",\"subscriber_id\":null,\"cron_id\":null}')",
      [now.into()],
    ))
    .await
    .unwrap();
  fixture
    .bootstrap
    .execute_unprepared("REVOKE INSERT ON task_outbox FROM konobangu_task_control_access")
    .await
    .unwrap();
  assert!(recorder::models::cron::Model::dispatch_due(queue, now).await.is_err());
  assert_eq!(count(&fixture, "SELECT count(*) FROM task_runs WHERE cron_id IS NOT NULL").await, 0);
  fixture
    .bootstrap
    .execute_unprepared("GRANT INSERT ON task_outbox TO konobangu_task_control_access")
    .await
    .unwrap();
  assert_eq!(recorder::models::cron::Model::dispatch_due(queue, now).await.unwrap(), 1);
  assert_eq!(count(&fixture, "SELECT count(*) FROM task_runs WHERE cron_id IS NOT NULL").await, 1);
  assert_eq!(state(&fixture, &id).await.1, 0);
  ctx.task().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
}

#[tokio::test]
async fn task_delivery_cron_dst_occurrences_and_edit_disable_polling() {
  let fixture = RlsFixture::new().await;
  let ctx = context(&fixture).await;
  let queue = ctx.task().queue_database().unwrap();
  let first = Utc.with_ymd_and_hms(2030, 11, 3, 5, 30, 0).unwrap();
  let second = Utc.with_ymd_and_hms(2030, 11, 3, 6, 30, 0).unwrap();
  fixture.bootstrap.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,r#"INSERT INTO cron (cron_expr,cron_timezone,next_run,enabled,system_task_cron) VALUES ('0 30 1 * * *','America/New_York',$1,true,'{"task_type":"test","task_id":"dst","subscriber_id":null,"cron_id":null}')"#,[first.into()])).await.unwrap();
  let (a, b) = tokio::join!(
    recorder::models::cron::Model::dispatch_due(queue, first),
    recorder::models::cron::Model::dispatch_due(queue, first)
  );
  assert_eq!(a.unwrap() + b.unwrap(), 1);
  fixture
    .bootstrap
    .execute_unprepared("UPDATE task_runs SET status='Done',done_at=clock_timestamp() WHERE cron_id IS NOT NULL")
    .await
    .unwrap();
  assert_eq!(recorder::models::cron::Model::dispatch_due(queue, second).await.unwrap(), 1);
  assert_eq!(
    count(&fixture, "SELECT count(DISTINCT scheduled_at_utc) FROM task_runs WHERE cron_id IS NOT NULL").await,
    2
  );
  fixture
    .bootstrap
    .execute_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "UPDATE cron SET enabled=false,next_run=$1",
      [second.into()],
    ))
    .await
    .unwrap();
  assert_eq!(recorder::models::cron::Model::dispatch_due(queue, second).await.unwrap(), 0);
  fixture
    .bootstrap
    .execute_unprepared("UPDATE task_runs SET status='Done'; UPDATE cron SET enabled=true,cron_expr='0 30 2 * * *',next_run='2030-03-09T07:30:00Z'")
    .await
    .unwrap();
  let spring = Utc.with_ymd_and_hms(2030, 3, 9, 7, 30, 0).unwrap();
  assert_eq!(recorder::models::cron::Model::dispatch_due(queue, spring).await.unwrap(), 1);
  let next = fixture
    .bootstrap
    .query_one_raw(Statement::from_string(DbBackend::Postgres, "SELECT next_run FROM cron"))
    .await
    .unwrap()
    .unwrap()
    .try_get::<chrono::DateTime<Utc>>("", "next_run")
    .unwrap();
  assert_eq!(next, Utc.with_ymd_and_hms(2030, 3, 11, 6, 30, 0).unwrap());
  assert_eq!(recorder::models::cron::Model::dispatch_due(queue, spring).await.unwrap(), 0);
  ctx.task().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
}
