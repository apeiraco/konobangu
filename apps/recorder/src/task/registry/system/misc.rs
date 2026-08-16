use std::sync::Arc;

use chrono::Utc;

use crate::{
  app::AppContextTrait,
  errors::RecorderResult,
  task::{AsyncTaskTrait, register_system_task_type},
};

register_system_task_type! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct EchoTask {
        pub task_id: String,
    }
}

#[async_trait::async_trait]
impl AsyncTaskTrait for EchoTask {
  async fn run_async(self, _ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
    tracing::info!("EchoTask {} start running at {}", self.task_id, Utc::now().to_rfc3339());

    Ok(())
  }
}
