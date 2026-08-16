mod registry;
mod rss;
mod subscription_episodes_feed;

use ::rss::Channel;
use async_trait::async_trait;
pub use registry::Feed;
pub use rss::{RssFeedItemTrait, RssFeedTrait};
use sea_orm::{ActiveValue, DeriveEntityModel, entity::prelude::*};
use serde::{Deserialize, Serialize};
pub use subscription_episodes_feed::SubscriptionEpisodesFeed;
use url::Url;

use crate::{
  app::AppContextTrait,
  errors::{RecorderError, RecorderResult},
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, DeriveActiveEnum, EnumIter, DeriveDisplay)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "feed_type")]
#[serde(rename_all = "snake_case")]
pub enum FeedType {
  #[sea_orm(string_value = "rss")]
  Rss,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, DeriveActiveEnum, EnumIter, DeriveDisplay)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "feed_source")]
#[serde(rename_all = "snake_case")]
pub enum FeedSource {
  #[sea_orm(string_value = "subscription_episode")]
  SubscriptionEpisode,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "feeds")]
pub struct Model {
  #[sea_orm(default_expr = "Expr::current_timestamp()")]
  pub created_at: DateTimeUtc,
  #[sea_orm(default_expr = "Expr::current_timestamp()")]
  pub updated_at: DateTimeUtc,
  #[sea_orm(primary_key)]
  pub id: i32,
  #[sea_orm(indexed)]
  pub token: String,
  #[sea_orm(indexed)]
  pub feed_type: FeedType,
  #[sea_orm(indexed)]
  pub feed_source: FeedSource,
  pub subscriber_id: Option<i32>,
  pub subscription_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::subscriptions::Entity",
    from = "Column::SubscriptionId",
    to = "super::subscriptions::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Subscription,
  #[sea_orm(
    belongs_to = "super::subscribers::Entity",
    from = "Column::SubscriberId",
    to = "super::subscribers::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Subscriber,
}

impl Related<super::subscriptions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Subscription.def()
  }
}

impl Related<super::subscribers::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Subscriber.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
  #[sea_orm(entity = "super::subscribers::Entity")]
  Subscriber,
  #[sea_orm(entity = "super::subscriptions::Entity")]
  Subscription,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
  async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
  where
    C: ConnectionTrait,
  {
    if insert && let ActiveValue::NotSet = self.token {
      let token = Uuid::now_v7().to_string();
      self.token = ActiveValue::Set(token);
    }
    Ok(self)
  }
}

impl Model {
  pub async fn find_rss_feed_by_token(ctx: &dyn AppContextTrait, token: &str, api_base: &Url, subscriber_id: i32) -> RecorderResult<Channel> {
    let transaction = crate::database::operation::begin_task_transaction(ctx.db().as_ref(), subscriber_id).await?;
    let db = &transaction;

    let feed_model = Entity::find()
      .filter(Column::Token.eq(token))
      .filter(Column::FeedType.eq(FeedType::Rss))
      .one(db)
      .await?
      .ok_or(RecorderError::from_entity_not_found::<Entity>())?;

    let feed = Feed::from_model(&transaction, feed_model).await?;

    transaction.rollback().await?;
    feed.into_rss_channel(ctx, api_base)
  }
}
