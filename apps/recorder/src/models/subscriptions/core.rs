use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;

use crate::{app::AppContextTrait, errors::RecorderResult, models::subscriptions};

#[async_trait]
pub trait SubscriptionTrait: Sized + Debug {
  fn get_subscriber_id(&self) -> i32;

  fn get_subscription_id(&self) -> i32;

  async fn sync_feeds_incremental(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()>;

  async fn sync_feeds_full(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()>;

  async fn sync_sources(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()>;

  fn try_from_model(model: &subscriptions::Model) -> RecorderResult<Self>;
}
