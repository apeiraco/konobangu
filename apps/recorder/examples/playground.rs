use std::{sync::Arc, time::Duration};

use recorder::{
  app::AppContextTrait,
  errors::RecorderResult,
  test_utils::{
    app::TestingAppContext,
    database::{TestingDatabaseServiceConfig, build_testing_database_service},
  },
};

async fn run_main() -> RecorderResult<()> {
  let app_ctx = {
    let db_service = build_testing_database_service(TestingDatabaseServiceConfig { auto_migrate: false }).await?;
    Arc::new(TestingAppContext::builder().db(db_service).build())
  };

  let db = app_ctx.db();

  recorder::migrations::legacy::initialize_queue(db.as_ref()).await?;

  println!("Owner queue migration completed");

  tokio::time::sleep(Duration::from_hours(1)).await;

  Ok(())
}

fn main() -> RecorderResult<()> {
  tokio::runtime::Builder::new_multi_thread().enable_all().build()?.block_on(run_main())
}
