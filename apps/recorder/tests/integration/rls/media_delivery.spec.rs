//! Ordinary-role persistent jobs and the production static controller.
#[path = "../support/jxl.rs"]
mod jxl_support;
use std::{io::Cursor, panic::AssertUnwindSafe, path::PathBuf, sync::Arc};

use axum::{Router, body::Body, http::Request};
use futures::FutureExt;
use recorder::{
  app::AppContextTrait,
  database::DatabaseService,
  media::{MediaService, derivative},
  storage::StorageService,
  task::{OptimizeImageTask, TaskConfig, TaskService},
  test_utils::app::TestingAppContext,
  web::controller::ControllerTrait,
};
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};
use tower::ServiceExt;

use super::support::RlsFixture;
fn png(value: u8) -> bytes::Bytes {
  let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(20, 20, image::Rgb([value, 40, 80])));
  let mut out = Cursor::new(Vec::new());
  image.write_to(&mut out, image::ImageFormat::Png).unwrap();
  out.into_inner().into()
}
#[tokio::test]
async fn media_delivery_persistent_legacy_replay_profiles_backfill_and_real_router() {
  let fixture = RlsFixture::new().await;
  let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../../temp/verification/fixtures/delivery")
    .join(uuid::Uuid::now_v7().to_string());
  std::fs::create_dir_all(&root).unwrap();
  let mut config = fixture.bootstrap.config.clone();
  config.uri = fixture.app_uri.clone();
  config.auto_migrate = false;
  let ctx = Arc::new(
    TestingAppContext::builder()
      .db(DatabaseService::from_config(config).await.unwrap())
      .storage(StorageService {
        data_dir: root.to_str().unwrap().into(),
        operator: StorageService::get_operator(root.to_str().unwrap()).unwrap(),
      })
      .media(MediaService::from_config(jxl_support::config()).await.unwrap())
      .build(),
  );
  ctx.set_task(
    TaskService::from_config_and_ctx(
      TaskConfig {
        queue_database_uri: Some(fixture.queue_uri.clone()),
        ..Default::default()
      },
      ctx.clone(),
    )
    .await
    .unwrap(),
  );
  let result=AssertUnwindSafe(async {
  let source="public/cover.png";ctx.storage().write(source,png(80)).await.unwrap();
  // Read an actual nonempty persisted old payload with no derivation version.
  let legacy=serde_json::json!({"task_type":"optimize_image","source_path":source,"target_path":"public/cover.webp","format_options":{"mime_type":"image/webp","quality":80}});
  let legacy:recorder::task::SystemTask=serde_json::from_value(legacy).unwrap();let id=ctx.task().add_system_task(legacy).await.unwrap();
  TaskService::execute(recorder::task::execution::TaskEnvelope{task_id:id.clone(),generation:1},apalis::prelude::Data::new(ctx.clone() as Arc<dyn AppContextTrait>)).await.unwrap();
  assert!(ctx.storage().exists("public/cover.webp").await.unwrap().is_some());assert_eq!(super::task_delivery::state(&fixture,&id).await.0,"Done");
  // Restore the old JSON into the durable row; reserializing first would
  // replace speed and fail to prove compatibility with stored bytes.
  let old_jxl=serde_json::json!({"task_type":"optimize_image","source_path":source,"target_path":"public/cover.jxl","format_options":{"mime_type":"image/jxl","quality":80,"speed":3}});
  assert!(serde_json::from_value::<recorder::task::SystemTask>(old_jxl.clone()).is_err());
  let explicit = recorder::task::OptimizeImageTask::builder().source_path(source.into()).target_path("public/cover.jxl".into()).format_options(recorder::media::EncodeImageOptions::Jxl(Default::default())).build();
  let old_id=ctx.task().add_system_task(explicit.into()).await.unwrap();
  fixture.bootstrap.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"UPDATE task_runs SET payload=$2 WHERE id=$1",[old_id.clone().into(),old_jxl.into()])).await.unwrap();
  let replay=TaskService::execute(recorder::task::execution::TaskEnvelope{task_id:old_id.clone(),generation:1},apalis::prelude::Data::new(ctx.clone() as Arc<dyn AppContextTrait>)).await;
  assert!(replay.is_err());
  assert!(ctx.storage().exists("public/cover.jxl").await.unwrap().is_none());
  let diagnostic = fixture.bootstrap.query_one_raw(Statement::from_sql_and_values(DbBackend::Postgres,"SELECT last_error FROM task_runs WHERE id=$1",[old_id.into()])).await.unwrap().unwrap();
  assert!(diagnostic.try_get::<String>("","last_error").unwrap().contains("format_options"));
  assert!(ctx.storage().exists(source).await.unwrap().is_some());
  let plans = ctx.media().derivative_plans();
  let expected_formats = recorder::media::MediaConfig::default().auto_optimize_formats;
  assert_eq!(plans.iter().map(|p| p.format).collect::<Vec<_>>(), expected_formats);
  assert_eq!(derivative::enqueue_missing(ctx.as_ref(),source).await.unwrap(),plans.len());
  assert_eq!(derivative::enqueue_missing(ctx.as_ref(),source).await.unwrap(),0);
  let pending = fixture.bootstrap.query_all_raw(Statement::from_string(DbBackend::Postgres,"SELECT id FROM task_runs WHERE status='Pending' ORDER BY created_at")).await.unwrap();
  assert_eq!(pending.len(), plans.len());
  for row in pending {
    let id:String=row.try_get("","id").unwrap();
    TaskService::execute(recorder::task::execution::TaskEnvelope{task_id:id.clone(),generation:1},apalis::prelude::Data::new(ctx.clone() as Arc<dyn AppContextTrait>)).await.unwrap();
    assert_eq!(super::task_delivery::state(&fixture,&id).await.0,"Done");
  }
  let manifest=ctx.storage().load_derivatives(source).await.unwrap();
  assert_eq!(manifest.entries.len(),plans.len());
  for format in expected_formats {
    let entry=manifest.entries.iter().find(|entry|entry.format==format).unwrap();
    let bytes=ctx.storage().read(&entry.path).await.unwrap().to_bytes();
    assert_eq!(derivative::hash(&bytes),entry.sha256);
    match format {
      recorder::media::AutoOptimizeImageFormat::Webp => assert_eq!(image::load_from_memory_with_format(&bytes,image::ImageFormat::WebP).unwrap().width(),20),
      #[cfg(feature="jxl")]
      recorder::media::AutoOptimizeImageFormat::Jxl => assert_eq!(jxl_support::decode(&bytes).dimensions(),(20,20)),
      _ => panic!("Unexpected default derivative"),
    }
  }
  assert_eq!(derivative::enqueue_missing(ctx.as_ref(),source).await.unwrap(),0);
  let controller=recorder::web::controller::r#static::create(ctx.clone()).await.unwrap();let router=controller.apply_to(Router::new()).with_state(ctx.clone() as Arc<dyn AppContextTrait>);
  let response=router.clone().oneshot(Request::builder().uri("/api/static/public/cover.png?optimize=accept").header("Accept","image/webp,image/png").body(Body::empty()).unwrap()).await.unwrap();assert_eq!(response.status(),200);assert_eq!(response.headers()["content-type"],"image/webp");let etag=response.headers()["etag"].clone();assert!(!etag.to_str().unwrap().starts_with("W/"));
  let response=router.clone().oneshot(Request::builder().method("HEAD").uri("/api/static/public/cover.png?optimize=accept").header("Accept","image/webp").body(Body::empty()).unwrap()).await.unwrap();assert_eq!(response.status(),200);assert_eq!(axum::body::to_bytes(response.into_body(),1000).await.unwrap().len(),0);
  for (accept,status) in [("image/png;q=bad",400),("application/jxl",406)] {let response=router.clone().oneshot(Request::builder().uri("/api/static/public/cover.png?optimize=accept").header("Accept",accept).body(Body::empty()).unwrap()).await.unwrap();assert_eq!(response.status(),status);}
  let response=router.clone().oneshot(Request::builder().uri("/api/static/public/cover.png?optimize=accept").header("Accept","image/webp").header("If-None-Match",etag.clone()).body(Body::empty()).unwrap()).await.unwrap();assert_eq!(response.status(),304);assert_eq!(response.headers()["cache-control"],"public, no-cache");
  let old_fingerprint=manifest.source_fingerprint.clone();ctx.storage().write(source,png(90)).await.unwrap();assert!(ctx.storage().load_derivatives(source).await.is_none());
  let plan=ctx.media().derivative_plans().pop().unwrap();let stale=OptimizeImageTask::builder().source_path(source.into()).target_path("unused".into()).format_options(plan.options).derivation_version(Some(1)).source_fingerprint(Some(old_fingerprint)).build();let id=ctx.task().add_system_task(stale.into()).await.unwrap();
  assert!(TaskService::execute(recorder::task::execution::TaskEnvelope{task_id:id.clone(),generation:1},apalis::prelude::Data::new(ctx.clone() as Arc<dyn AppContextTrait>)).await.is_err());assert!(ctx.storage().load_derivatives(source).await.is_none());assert!(ctx.storage().exists(source).await.unwrap().is_some());
  // Independent workers for the same full source path must merge entries
  // under the resource lock, even when both encoded before publication.
  let concurrent="public/concurrent.png";ctx.storage().write(concurrent,png(70)).await.unwrap();
  let fingerprint=derivative::fingerprint(&ctx.storage().stat(concurrent).await.unwrap()).unwrap();
  let mut ids=Vec::new();
  for options in [recorder::media::EncodeImageOptions::Webp(recorder::media::EncodeWebpOptions{quality:Some(80.0)}),recorder::media::EncodeImageOptions::Avif(recorder::media::EncodeAvifOptions{quality:Some(80),speed:Some(6),threads:Some(1)})] {
    let task=OptimizeImageTask::builder().source_path(concurrent.into()).target_path("unused".into()).format_options(options).derivation_version(Some(1)).source_fingerprint(Some(fingerprint.clone())).build();
    ids.push(ctx.task().add_system_task(task.into()).await.unwrap());
  }
  let lock=fixture.bootstrap.begin().await.unwrap();derivative::lock_resource(&lock,concurrent).await.unwrap();
  let mut workers=Vec::new();
  for id in &ids {let id=id.clone();let ctx=ctx.clone();workers.push(tokio::spawn(async move {TaskService::execute(recorder::task::execution::TaskEnvelope{task_id:id,generation:1},apalis::prelude::Data::new(ctx as Arc<dyn AppContextTrait>)).await}));}
  let ready=tokio::time::timeout(std::time::Duration::from_secs(10),async {
    loop {
      let directory=root.join("public/concurrent.png.derived");
      let ready=std::fs::read_dir(&directory).map(|entries|entries.filter_map(Result::ok).filter(|entry|entry.path().extension().is_some_and(|ext|ext=="part")).count()).unwrap_or(0);
      if ready==2 {break;}
      tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
  }).await;
  if ready.is_err() {
    for worker in &workers {worker.abort();}
    lock.rollback().await.unwrap();
    for worker in workers {let _=worker.await;}
    panic!("Concurrent image workers did not reach publication");
  }
  assert!(ctx.storage().load_derivatives(concurrent).await.is_none());
  lock.commit().await.unwrap();
  for worker in workers {worker.await.unwrap().unwrap();}
  let merged=ctx.storage().load_derivatives(concurrent).await.unwrap();assert_eq!(merged.entries.len(),2);
  for entry in &merged.entries {let bytes=ctx.storage().read(&entry.path).await.unwrap().to_bytes();assert_eq!(derivative::hash(&bytes),entry.sha256);}
  assert!(std::fs::read_dir(root.join("public/concurrent.png.derived")).unwrap().all(|entry|entry.unwrap().path().extension().is_none_or(|ext|ext!="part")));
  // A late result that has lost its fence cannot publish even though encoding
  // already finished. Its owned temporary file must disappear.
  let late_source="public/late.png";ctx.storage().write(late_source,png(60)).await.unwrap();
  let late=OptimizeImageTask::builder().source_path(late_source.into()).target_path("unused".into()).format_options(ctx.media().derivative_plans()[0].options.clone()).derivation_version(Some(1)).source_fingerprint(derivative::fingerprint(&ctx.storage().stat(late_source).await.unwrap())).build();
  let late_id=ctx.task().add_system_task(late.clone().into()).await.unwrap();
  let token=uuid::Uuid::now_v7();
  fixture.bootstrap.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"UPDATE task_runs SET status='Running',execution_token=$2,execution_lease_until=clock_timestamp()+INTERVAL '120 seconds',cancel_requested_at=clock_timestamp() WHERE id=$1",[late_id.clone().into(),token.into()])).await.unwrap();
  let fence=recorder::task::execution::TaskFence{envelope:recorder::task::execution::TaskEnvelope{task_id:late_id,generation:1},token};
  use recorder::task::AsyncTaskTrait;
  assert!(recorder::task::execution::CURRENT_FENCE.scope(fence,late.run_async(ctx.clone())).await.is_err());
  assert!(ctx.storage().load_derivatives(late_source).await.is_none());assert!(ctx.storage().exists(late_source).await.unwrap().is_some());
  assert!(std::fs::read_dir(root.join("public/late.png.derived")).unwrap().all(|entry|entry.unwrap().path().extension().is_none_or(|ext|ext!="part")));
  let expiry_source="public/expired.png";ctx.storage().write(expiry_source,png(55)).await.unwrap();
  let expiry=OptimizeImageTask::builder().source_path(expiry_source.into()).target_path("unused".into()).format_options(ctx.media().derivative_plans()[0].options.clone()).derivation_version(Some(1)).source_fingerprint(derivative::fingerprint(&ctx.storage().stat(expiry_source).await.unwrap())).build();
  let expiry_id=ctx.task().add_system_task(expiry.clone().into()).await.unwrap();let token=uuid::Uuid::now_v7();
  fixture.bootstrap.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"UPDATE task_runs SET status='Running',execution_token=$2,execution_lease_until=clock_timestamp()+INTERVAL '2 seconds' WHERE id=$1",[expiry_id.clone().into(),token.into()])).await.unwrap();
  let held=fixture.bootstrap.begin().await.unwrap();derivative::lock_resource(&held,expiry_source).await.unwrap();
  let fence=recorder::task::execution::TaskFence{envelope:recorder::task::execution::TaskEnvelope{task_id:expiry_id,generation:1},token};
  let expiry_ctx=ctx.clone();let worker=tokio::spawn(async move{recorder::task::execution::CURRENT_FENCE.scope(fence,expiry.run_async(expiry_ctx)).await});
  let ready=tokio::time::timeout(std::time::Duration::from_secs(5),async {
    loop {if std::fs::read_dir(root.join("public/expired.png.derived")).is_ok_and(|mut entries|entries.next().is_some()){break;}
      tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
  }).await;
  if ready.is_err(){worker.abort();held.rollback().await.unwrap();let _=worker.await;panic!("Image worker did not stage its result");}
  tokio::time::sleep(std::time::Duration::from_millis(2100)).await;
  held.commit().await.unwrap();assert!(worker.await.unwrap().is_err());
  assert!(ctx.storage().load_derivatives(expiry_source).await.is_none());assert!(ctx.storage().exists(expiry_source).await.unwrap().is_some());
  assert!(std::fs::read_dir(root.join("public/expired.png.derived")).unwrap().next().is_none());
  let other="public/cover.jpg";ctx.storage().write(other,png(70)).await.unwrap();assert_eq!(derivative::enqueue_missing(ctx.as_ref(),other).await.unwrap(),plans.len());
  let private=OptimizeImageTask::builder().source_path("subscribers/1002/a.png".into()).target_path("public/stolen.webp".into()).format_options(recorder::media::EncodeImageOptions::Webp(Default::default())).subscriber_id(Some(1001)).build();assert!(private.run_async(ctx.clone()).await.is_err());
 }).catch_unwind().await;
  ctx.task().queue_database().unwrap().clone().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
  std::fs::remove_dir_all(root).unwrap();
  if let Err(panic) = result {
    std::panic::resume_unwind(panic);
  }
}
