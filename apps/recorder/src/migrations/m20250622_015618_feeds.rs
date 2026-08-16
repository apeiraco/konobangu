use async_trait::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use crate::{
  migrations::defs::{CustomSchemaManagerExt, Feeds, GeneralIds, Subscribers, Subscriptions, table_auto_z},
  models::feeds::{FeedSource, FeedSourceEnum, FeedType, FeedTypeEnum},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    create_postgres_enum_for_active_enum!(manager, FeedTypeEnum, FeedType::Rss).await?;
    create_postgres_enum_for_active_enum!(manager, FeedSourceEnum, FeedSource::SubscriptionEpisode).await?;

    manager
      .create_table(
        table_auto_z(Feeds::Table)
          .col(pk_auto(Feeds::Id))
          .col(text(Feeds::Token))
          .col(enumeration(Feeds::FeedType, FeedTypeEnum, FeedType::iden_values()))
          .col(enumeration(Feeds::FeedSource, FeedSourceEnum, FeedSource::iden_values()).not_null())
          .col(integer_null(Feeds::SubscriberId))
          .col(integer_null(Feeds::SubscriptionId))
          .index(
            Index::create()
              .if_not_exists()
              .name("idx_feeds_token")
              .table(Feeds::Table)
              .col(Feeds::Token)
              .unique(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_feeds_subscriber_id")
              .from(Feeds::Table, Feeds::SubscriberId)
              .to(Subscribers::Table, Subscribers::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_feeds_subscription_id")
              .from(Feeds::Table, Feeds::SubscriptionId)
              .to(Subscriptions::Table, Subscriptions::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_postgres_auto_update_ts_trigger_for_col(Feeds::Table, GeneralIds::UpdatedAt)
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_postgres_auto_update_ts_trigger_for_col(Feeds::Table, GeneralIds::UpdatedAt)
      .await?;

    manager.drop_table(Table::drop().if_exists().table(Feeds::Table).to_owned()).await?;

    manager.drop_postgres_enum_for_active_enum(FeedTypeEnum).await?;

    manager.drop_postgres_enum_for_active_enum(FeedSourceEnum).await?;

    Ok(())
  }
}
