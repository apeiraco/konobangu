use async_trait::async_trait;
use sea_orm::{ActiveValue, entity::prelude::*, sea_query::OnConflict};
use serde::{Deserialize, Serialize};

use crate::{app::AppContextTrait, errors::RecorderResult};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "subscription_episode")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i32,
  pub subscriber_id: i32,
  pub subscription_id: i32,
  pub episode_id: i32,
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
    belongs_to = "super::episodes::Entity",
    from = "Column::EpisodeId",
    to = "super::episodes::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Episode,
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

impl Related<super::episodes::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Episode.def()
  }
}

impl Related<super::subscribers::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Subscriber.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
  #[sea_orm(entity = "super::subscriptions::Entity")]
  Subscription,
  #[sea_orm(entity = "super::episodes::Entity")]
  Episode,
  #[sea_orm(entity = "super::subscribers::Entity")]
  Subscriber,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl Model {
  pub async fn add_episodes_for_subscription(
    ctx: &dyn AppContextTrait,
    episode_ids: impl Iterator<Item = i32>,
    subscriber_id: i32,
    subscription_id: i32,
  ) -> RecorderResult<()> {
    let transaction = crate::database::operation::begin_task_transaction(ctx.db().as_ref(), subscriber_id).await?;
    let db = &transaction;

    let active_models = episode_ids
      .map(|episode_id| ActiveModel {
        episode_id: ActiveValue::Set(episode_id),
        subscription_id: ActiveValue::Set(subscription_id),
        subscriber_id: ActiveValue::Set(subscriber_id),
        ..Default::default()
      })
      .collect::<Vec<_>>();

    if active_models.is_empty() {
      return Ok(());
    }

    crate::database::operation::require_visible::<super::subscriptions::Entity>(db, subscription_id).await?;
    for model in &active_models {
      crate::database::operation::require_visible::<super::episodes::Entity>(db, *model.episode_id.as_ref()).await?;
    }

    Entity::insert_many(active_models)
      .on_conflict(OnConflict::columns([Column::SubscriptionId, Column::EpisodeId]).do_nothing().to_owned())
      .exec_without_returning(db)
      .await?;

    crate::database::operation::commit_task_transaction(transaction).await?;
    Ok(())
  }
}
