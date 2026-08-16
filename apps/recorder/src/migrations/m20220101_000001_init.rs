use async_trait::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::defs::{Bangumi, CustomSchemaManagerExt, Episodes, GeneralIds, Subscribers, SubscriptionBangumi, SubscriptionEpisode, Subscriptions, table_auto_z};
use crate::models::{
  subscribers::SEED_SUBSCRIBER,
  subscriptions::{self, SubscriptionCategoryEnum},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.create_postgres_auto_update_ts_fn_for_col(GeneralIds::UpdatedAt).await?;

    manager
      .create_table(
        table_auto_z(Subscribers::Table)
          .col(pk_auto(Subscribers::Id))
          .col(string(Subscribers::DisplayName))
          .col(json_binary_null(Subscribers::BangumiConf))
          .to_owned(),
      )
      .await?;

    manager
      .create_postgres_auto_update_ts_trigger_for_col(Subscribers::Table, GeneralIds::UpdatedAt)
      .await?;

    manager
      .exec_stmt(
        Query::insert()
          .into_table(Subscribers::Table)
          .columns([Subscribers::DisplayName])
          .values_panic([SEED_SUBSCRIBER.into()])
          .to_owned(),
      )
      .await?;

    create_postgres_enum_for_active_enum!(
      manager,
      subscriptions::SubscriptionCategoryEnum,
      subscriptions::SubscriptionCategory::MikanSubscriber,
      subscriptions::SubscriptionCategory::MikanBangumi,
      subscriptions::SubscriptionCategory::MikanSeason
    )
    .await?;

    manager
      .create_table(
        table_auto_z(Subscriptions::Table)
          .col(pk_auto(Subscriptions::Id))
          .col(string(Subscriptions::DisplayName))
          .col(integer(Subscriptions::SubscriberId))
          .col(text(Subscriptions::SourceUrl))
          .col(boolean(Subscriptions::Enabled))
          .col(enumeration(
            Subscriptions::Category,
            subscriptions::SubscriptionCategoryEnum,
            subscriptions::SubscriptionCategory::iden_values(),
          ))
          .foreign_key(
            ForeignKey::create()
              .name("fk_subscriptions_subscriber_id")
              .from(Subscriptions::Table, Subscriptions::SubscriberId)
              .to(Subscribers::Table, Subscribers::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_postgres_auto_update_ts_trigger_for_col(Subscriptions::Table, GeneralIds::UpdatedAt)
      .await?;

    manager
      .create_table(
        table_auto_z(Bangumi::Table)
          .col(pk_auto(Bangumi::Id))
          .col(text_null(Bangumi::MikanBangumiId))
          .col(integer(Bangumi::SubscriberId))
          .col(text(Bangumi::DisplayName))
          .col(text(Bangumi::OriginName))
          .col(integer(Bangumi::Season))
          .col(text_null(Bangumi::SeasonRaw))
          .col(text_null(Bangumi::Fansub))
          .col(text_null(Bangumi::MikanFansubId))
          .col(json_binary_null(Bangumi::Filter))
          .col(text_null(Bangumi::RssLink))
          .col(text_null(Bangumi::PosterLink))
          .col(text_null(Bangumi::OriginPosterLink))
          .col(text_null(Bangumi::SavePath))
          .col(text_null(Bangumi::Homepage))
          .foreign_key(
            ForeignKey::create()
              .name("fk_bangumi_subscriber_id")
              .from(Bangumi::Table, Bangumi::SubscriberId)
              .to(Subscribers::Table, Subscribers::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .index(
            Index::create()
              .if_not_exists()
              .name("idx_bangumi_mikan_bangumi_id_mikan_fansub_id_subscriber_id")
              .table(Bangumi::Table)
              .col(Bangumi::MikanBangumiId)
              .col(Bangumi::MikanFansubId)
              .col(Bangumi::SubscriberId)
              .unique(),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .if_not_exists()
          .name("idx_bangumi_mikan_bangumi_id")
          .table(Bangumi::Table)
          .col(Bangumi::MikanBangumiId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .if_not_exists()
          .name("idx_bangumi_mikan_fansub_id")
          .table(Bangumi::Table)
          .col(Bangumi::MikanFansubId)
          .to_owned(),
      )
      .await?;

    manager
      .create_postgres_auto_update_ts_trigger_for_col(Bangumi::Table, GeneralIds::UpdatedAt)
      .await?;

    manager
      .create_table(
        table_auto_z(SubscriptionBangumi::Table)
          .col(pk_auto(SubscriptionBangumi::Id))
          .col(integer(SubscriptionBangumi::SubscriberId))
          .col(integer(SubscriptionBangumi::SubscriptionId))
          .col(integer(SubscriptionBangumi::BangumiId))
          .foreign_key(
            ForeignKey::create()
              .name("fk_subscription_bangumi_subscription_id")
              .from(SubscriptionBangumi::Table, SubscriptionBangumi::SubscriptionId)
              .to(Subscriptions::Table, Subscriptions::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_subscription_bangumi_bangumi_id")
              .from(SubscriptionBangumi::Table, SubscriptionBangumi::BangumiId)
              .to(Bangumi::Table, Bangumi::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_subscription_bangumi_subscriber_id")
              .from(SubscriptionBangumi::Table, SubscriptionBangumi::SubscriberId)
              .to(Subscribers::Table, Subscribers::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .index(
            Index::create()
              .if_not_exists()
              .name("constraint_subscription_bangumi_subscription_id_bangumi_id")
              .table(SubscriptionBangumi::Table)
              .col(SubscriptionBangumi::SubscriptionId)
              .col(SubscriptionBangumi::BangumiId)
              .unique(),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .if_not_exists()
          .name("idx_subscription_bangumi_subscriber_id")
          .table(SubscriptionBangumi::Table)
          .col(SubscriptionBangumi::SubscriberId)
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        table_auto_z(Episodes::Table)
          .col(pk_auto(Episodes::Id))
          .col(text_null(Episodes::MikanEpisodeId))
          .col(text(Episodes::OriginName))
          .col(text(Episodes::DisplayName))
          .col(integer(Episodes::BangumiId))
          .col(integer(Episodes::SubscriberId))
          .col(text_null(Episodes::SavePath))
          .col(text_null(Episodes::Resolution))
          .col(integer(Episodes::Season))
          .col(text_null(Episodes::SeasonRaw))
          .col(text_null(Episodes::Fansub))
          .col(text_null(Episodes::PosterLink))
          .col(text_null(Episodes::OriginPosterLink))
          .col(integer(Episodes::EpisodeIndex))
          .col(text_null(Episodes::Homepage))
          .col(text_null(Episodes::Subtitle))
          .col(text_null(Episodes::Source))
          .foreign_key(
            ForeignKey::create()
              .name("fk_episodes_bangumi_id")
              .from(Episodes::Table, Episodes::BangumiId)
              .to(Bangumi::Table, Bangumi::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_episodes_subscriber_id")
              .from(Episodes::Table, Episodes::SubscriberId)
              .to(Subscribers::Table, Subscribers::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .index(
            Index::create()
              .if_not_exists()
              .name("idx_episodes_mikan_episode_id_subscriber_id")
              .table(Episodes::Table)
              .col(Episodes::MikanEpisodeId)
              .col(Episodes::SubscriberId)
              .unique(),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .if_not_exists()
          .name("idx_episodes_mikan_episode_id")
          .table(Episodes::Table)
          .col(Episodes::MikanEpisodeId)
          .to_owned(),
      )
      .await?;

    manager
      .create_postgres_auto_update_ts_trigger_for_col(Episodes::Table, GeneralIds::UpdatedAt)
      .await?;

    manager
      .create_table(
        table_auto_z(SubscriptionEpisode::Table)
          .col(pk_auto(SubscriptionEpisode::Id))
          .col(integer(SubscriptionEpisode::SubscriptionId))
          .col(integer(SubscriptionEpisode::EpisodeId))
          .col(integer(SubscriptionEpisode::SubscriberId))
          .foreign_key(
            ForeignKey::create()
              .name("fk_subscription_episode_subscription_id")
              .from(SubscriptionEpisode::Table, SubscriptionEpisode::SubscriptionId)
              .to(Subscriptions::Table, Subscriptions::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_subscription_episode_episode_id")
              .from(SubscriptionEpisode::Table, SubscriptionEpisode::EpisodeId)
              .to(Episodes::Table, Episodes::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_subscription_episode_subscriber_id")
              .from(SubscriptionEpisode::Table, SubscriptionEpisode::SubscriberId)
              .to(Subscribers::Table, Subscribers::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .index(
            Index::create()
              .if_not_exists()
              .name("constraint_subscription_episode_subscription_id_episode_id")
              .table(SubscriptionEpisode::Table)
              .col(SubscriptionEpisode::SubscriptionId)
              .col(SubscriptionEpisode::EpisodeId)
              .unique(),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .if_not_exists()
          .name("idx_subscription_episode_subscriber_id")
          .table(SubscriptionEpisode::Table)
          .col(SubscriptionEpisode::SubscriberId)
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_index(
        Index::drop()
          .if_exists()
          .name("idx_subscription_episode_subscriber_id")
          .table(SubscriptionBangumi::Table)
          .to_owned(),
      )
      .await?;

    manager
      .drop_table(Table::drop().if_exists().table(SubscriptionEpisode::Table).to_owned())
      .await?;

    manager
      .drop_postgres_auto_update_ts_trigger_for_col(Episodes::Table, GeneralIds::UpdatedAt)
      .await?;

    manager.drop_table(Table::drop().if_exists().table(Episodes::Table).to_owned()).await?;

    manager
      .drop_index(
        Index::drop()
          .if_exists()
          .name("idx_subscription_bangumi_subscriber_id")
          .table(SubscriptionBangumi::Table)
          .to_owned(),
      )
      .await?;

    manager
      .drop_table(Table::drop().if_exists().table(SubscriptionBangumi::Table).to_owned())
      .await?;

    manager
      .drop_postgres_auto_update_ts_trigger_for_col(Bangumi::Table, GeneralIds::UpdatedAt)
      .await?;

    manager.drop_table(Table::drop().if_exists().table(Bangumi::Table).to_owned()).await?;

    manager
      .drop_postgres_auto_update_ts_trigger_for_col(Subscriptions::Table, GeneralIds::UpdatedAt)
      .await?;

    manager.drop_table(Table::drop().if_exists().table(Subscriptions::Table).to_owned()).await?;

    manager
      .drop_postgres_auto_update_ts_trigger_for_col(Subscribers::Table, GeneralIds::UpdatedAt)
      .await?;

    manager.drop_table(Table::drop().if_exists().table(Subscribers::Table).to_owned()).await?;

    manager.drop_postgres_enum_for_active_enum(subscriptions::SubscriptionCategoryEnum).await?;

    manager.drop_postgres_enum_for_active_enum(SubscriptionCategoryEnum).await?;

    manager.drop_postgres_auto_update_ts_fn_for_col(GeneralIds::UpdatedAt).await?;

    Ok(())
  }
}
