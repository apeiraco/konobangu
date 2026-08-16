use std::sync::Arc;

use async_trait::async_trait;
use futures::{Stream, StreamExt, pin_mut};
use serde::{Serialize, de::DeserializeOwned};

use crate::{app::AppContextTrait, errors::RecorderResult};

pub const SYSTEM_TASK_APALIS_NAME: &str = "system_task";
pub const SUBSCRIBER_TASK_APALIS_NAME: &str = "subscriber_task";
pub const SETUP_APALIS_JOBS_EXTRA_FOREIGN_KEYS_FUNCTION_NAME: &str = "setup_apalis_jobs_extra_foreign_keys";
pub const SETUP_APALIS_JOBS_EXTRA_FOREIGN_KEYS_TRIGGER_NAME: &str = "setup_apalis_jobs_extra_foreign_keys_trigger";

#[async_trait]
pub trait AsyncTaskTrait: Serialize + DeserializeOwned + Sized {
  async fn run_async(self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()>;
}

pub trait StreamTaskTrait {
  type Yield: Serialize + DeserializeOwned + Send;

  fn run_stream(self, ctx: Arc<dyn AppContextTrait>) -> impl Stream<Item = RecorderResult<Self::Yield>> + Send;
}

#[async_trait]
impl<T> AsyncTaskTrait for T
where
  T: StreamTaskTrait + Serialize + DeserializeOwned + Sized + Send,
{
  async fn run_async(self, _ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
    let s = self.run_stream(_ctx);

    pin_mut!(s);

    while let Some(item) = s.next().await {
      item?;
    }

    Ok(())
  }
}

pub trait SystemTaskTrait: AsyncTaskTrait {
  type InputType: Serialize + DeserializeOwned + Sized + Send;

  fn get_subscriber_id(&self) -> Option<i32>;

  fn set_subscriber_id(&mut self, subscriber_id: Option<i32>);

  fn get_cron_id(&self) -> Option<i32>;

  fn set_cron_id(&mut self, cron_id: Option<i32>);

  fn from_input(input: Self::InputType, subscriber_id: Option<i32>) -> Self;
}

pub trait SubscriberTaskTrait: AsyncTaskTrait {
  type InputType: Serialize + DeserializeOwned + Sized + Send;

  fn get_subscriber_id(&self) -> i32;

  fn set_subscriber_id(&mut self, subscriber_id: i32);

  fn get_cron_id(&self) -> Option<i32>;

  fn set_cron_id(&mut self, cron_id: Option<i32>);

  fn from_input(input: Self::InputType, subscriber_id: i32) -> Self;
}
