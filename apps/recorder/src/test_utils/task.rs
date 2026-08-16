use std::sync::Arc;

use crate::{
  app::AppContextTrait,
  errors::RecorderResult,
  task::{TaskConfig, TaskService},
};

pub async fn build_testing_task_service(config: Option<TaskConfig>, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<TaskService> {
  let config = config.unwrap_or_default();
  #[cfg(feature = "testcontainers")]
  let config = {
    let mut config = config;
    if config.queue_database_uri.is_none() {
      config.queue_database_uri = Some(ctx.db().config.uri.clone());
    }
    config
  };
  let task_service = TaskService::from_config_and_ctx(config, ctx).await?;

  Ok(task_service)
}
