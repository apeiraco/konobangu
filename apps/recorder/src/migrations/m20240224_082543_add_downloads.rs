use async_trait::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::defs::*;
use crate::models::{
  downloaders::{DownloaderCategory, DownloaderCategoryEnum},
  downloads::{DownloadMime, DownloadMimeEnum, DownloadStatus, DownloadStatusEnum},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    create_postgres_enum_for_active_enum!(manager, DownloaderCategoryEnum, DownloaderCategory::QBittorrent, DownloaderCategory::Dandanplay).await?;

    manager
      .create_table(
        table_auto_z(Downloaders::Table)
          .col(pk_auto(Downloaders::Id))
          .col(text(Downloaders::Endpoint))
          .col(string_null(Downloaders::Username))
          .col(string_null(Downloaders::Password))
          .col(enumeration(Downloaders::Category, DownloaderCategoryEnum, DownloaderCategory::iden_values()))
          .col(text(Downloaders::SavePath))
          .col(integer(Downloaders::SubscriberId))
          .foreign_key(
            ForeignKey::create()
              .name("fk_downloader_subscriber_id")
              .from_tbl(Downloaders::Table)
              .from_col(Downloaders::SubscriberId)
              .to_tbl(Subscribers::Table)
              .to_col(Subscribers::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_postgres_auto_update_ts_trigger_for_col(Downloaders::Table, GeneralIds::UpdatedAt)
      .await?;

    create_postgres_enum_for_active_enum!(manager, DownloadMimeEnum, DownloadMime::BitTorrent, DownloadMime::OctetStream).await?;

    create_postgres_enum_for_active_enum!(
      manager,
      DownloadStatusEnum,
      DownloadStatus::Downloading,
      DownloadStatus::Paused,
      DownloadStatus::Pending,
      DownloadStatus::Completed,
      DownloadStatus::Failed,
      DownloadStatus::Deleted
    )
    .await?;

    manager
      .create_table(
        table_auto_z(Downloads::Table)
          .col(pk_auto(Downloads::Id))
          .col(string(Downloads::OriginName))
          .col(string(Downloads::DisplayName))
          .col(integer(Downloads::SubscriberId))
          .col(integer(Downloads::DownloaderId))
          .col(integer(Downloads::EpisodeId))
          .col(enumeration(Downloads::Status, DownloadStatusEnum, DownloadStatus::iden_values()))
          .col(enumeration(Downloads::Mime, DownloadMimeEnum, DownloadMime::iden_values()))
          .col(big_integer(Downloads::AllSize))
          .col(big_integer(Downloads::CurrSize))
          .col(text(Downloads::Url))
          .col(text_null(Downloads::Homepage))
          .col(text_null(Downloads::SavePath))
          .foreign_key(
            ForeignKey::create()
              .name("fk_downloads_subscriber_id")
              .from_tbl(Downloads::Table)
              .from_col(Downloads::SubscriberId)
              .to_tbl(Subscribers::Table)
              .to_col(Subscribers::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_downloads_downloader_id")
              .from_tbl(Downloads::Table)
              .from_col(Downloads::DownloaderId)
              .to_tbl(Downloaders::Table)
              .to_col(Downloaders::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_downloads_episode_id")
              .from_tbl(Downloads::Table)
              .from_col(Downloads::EpisodeId)
              .to_tbl(Episodes::Table)
              .to_col(Episodes::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_postgres_auto_update_ts_trigger_for_col(Downloads::Table, GeneralIds::UpdatedAt)
      .await?;

    manager
      .create_index(
        Index::create()
          .if_not_exists()
          .name("idx_downloads_url")
          .table(Downloads::Table)
          .col(Downloads::Url)
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_postgres_auto_update_ts_trigger_for_col(Downloads::Table, GeneralIds::UpdatedAt)
      .await?;

    manager.drop_table(Table::drop().if_exists().table(Downloads::Table).to_owned()).await?;

    manager.drop_postgres_enum_for_active_enum(DownloadMimeEnum).await?;

    manager.drop_postgres_enum_for_active_enum(DownloadStatusEnum).await?;

    manager
      .drop_postgres_auto_update_ts_trigger_for_col(Downloaders::Table, GeneralIds::UpdatedAt)
      .await?;

    manager.drop_table(Table::drop().if_exists().table(Downloaders::Table).to_owned()).await?;

    manager.drop_postgres_enum_for_active_enum(DownloaderCategoryEnum).await?;

    Ok(())
  }
}
