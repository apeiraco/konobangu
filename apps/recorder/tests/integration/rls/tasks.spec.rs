use std::sync::Arc;

use recorder::{
  database::{DatabaseService, operation::IdentityOperation},
  models::credential_3rd,
  task::{AsyncTaskTrait, SubscriberTask, SubscriberTaskTrait},
  test_utils::{app::TestingAppContext, crypto::build_testing_crypto_service},
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, DbBackend, EntityTrait, Statement};

use super::support::{RlsFixture, SUBSCRIBER_A, SUBSCRIBER_B};

#[tokio::test]
async fn user_task_owner_database_stages_and_cancellation_fail_closed() {
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
  let input =
    serde_json::from_value(serde_json::json!({ "taskType": "sync_one_subscription_sources", "subscriptionId": 2, "subscriberId": SUBSCRIBER_B })).unwrap();
  let task = SubscriberTask::from_input(input, SUBSCRIBER_A);
  assert_eq!(task.get_subscriber_id(), SUBSCRIBER_A, "payload owner is never authoritative");
  // B's subscription is rejected before the task can access network services.
  assert!(task.clone().run_async(ctx.clone()).await.is_err());
  let prepared = || async {
    Ok(recorder::models::bangumi::ActiveModel {
      display_name: Set("A task result".into()),
      origin_name: Set("A task result".into()),
      bangumi_type: Set(recorder::models::bangumi::BangumiType::Mikan),
      season: Set(1),
      mikan_bangumi_id: Set(Some("fixture-bangumi".into())),
      mikan_fansub_id: Set(Some("fixture-group".into())),
      ..Default::default()
    })
  };
  let hash = recorder::extract::mikan::MikanBangumiHash {
    mikan_bangumi_id: "fixture-bangumi".into(),
    mikan_fansub_id: "fixture-group".into(),
  };
  let model = recorder::models::bangumi::Model::get_or_insert_from_mikan(ctx.as_ref(), hash.clone(), SUBSCRIBER_A, 1, prepared)
    .await
    .unwrap();
  assert_eq!(model.subscriber_id, SUBSCRIBER_A);
  assert!(
    recorder::models::bangumi::Model::get_or_insert_from_mikan(ctx.as_ref(), hash, SUBSCRIBER_A, 2, prepared)
      .await
      .is_err()
  );
  assert!(
    recorder::models::subscription_bangumi::Model::add_bangumis_for_subscription(ctx.as_ref(), [model.id].into_iter(), SUBSCRIBER_B, 2)
      .await
      .is_err()
  );
  let persisted = fixture
    .bootstrap
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT count(*) FROM subscription_bangumi WHERE subscriber_id = 1001 AND subscription_id = 1",
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get_by_index::<i64>(0)
    .unwrap();
  assert_eq!(persisted, 1, "actual task database stages commit the resource and association atomically");
  let operation = IdentityOperation::begin(&fixture.app, SUBSCRIBER_A).await.unwrap();
  assert!(
    operation
      .run(move |db| Box::pin(async move { recorder::task::operation::enqueue(db, recorder::task::SUBSCRIBER_TASK_APALIS_NAME, &task).await }))
      .await
      .is_err()
  );
  operation.finish(false).await.unwrap();
  let operation = IdentityOperation::begin(&fixture.app, SUBSCRIBER_A).await.unwrap();
  operation
    .run(|db| {
      Box::pin(async move {
        credential_3rd::ActiveModel {
          username: Set(Some("cancelled-write".into())),
          credential_type: Set(credential_3rd::Credential3rdType::Mikan),
          subscriber_id: Set(SUBSCRIBER_A),
          ..Default::default()
        }
        .insert(db)
        .await
      })
    })
    .await
    .unwrap();
  // Cancellation drops the last owner without commit; pool reuse waits for
  // rollback.
  drop(operation);
  let next = IdentityOperation::begin(&fixture.app, SUBSCRIBER_B).await.unwrap();
  assert!(
    next
      .run(|db| Box::pin(async move { credential_3rd::Entity::find().all(db).await }))
      .await
      .unwrap()
      .is_empty()
  );
  next.finish(false).await.unwrap();
  let count = fixture
    .bootstrap
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT count(*) FROM credential3rd WHERE username = 'cancelled-write'",
    ))
    .await
    .unwrap()
    .unwrap();
  assert_eq!(count.try_get_by_index::<i64>(0).unwrap(), 0);
  fixture.close().await;
}
