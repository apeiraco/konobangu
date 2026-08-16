use async_trait::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use crate::{
  migrations::defs::{Bangumi, CustomSchemaManagerExt, Episodes},
  models::{
    bangumi::{BangumiType, BangumiTypeEnum},
    episodes::{EpisodeType, EpisodeTypeEnum},
  },
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let db = manager.get_connection();

    create_postgres_enum_for_active_enum!(manager, EpisodeTypeEnum, EpisodeType::Mikan).await?;

    {
      create_postgres_enum_for_active_enum!(manager, BangumiTypeEnum, BangumiType::Mikan).await?;
      manager
        .alter_table(
          Table::alter()
            .table(Bangumi::Table)
            .add_column_if_not_exists(enumeration_null(Bangumi::BangumiType, BangumiTypeEnum, BangumiType::iden_values()))
            .to_owned(),
        )
        .await?;

      db.execute_unprepared(&format!(
        r#"ALTER TABLE {bangumi} DROP COLUMN IF EXISTS {save_path}"#,
        bangumi = Bangumi::Table.to_string(),
        save_path = Bangumi::SavePath.to_string(),
      ))
      .await?;

      manager
        .exec_stmt(
          UpdateStatement::new()
            .table(Bangumi::Table)
            .value(Bangumi::BangumiType, BangumiType::Mikan.as_enum(BangumiTypeEnum))
            .and_where(Expr::col(Bangumi::BangumiType).is_null())
            .and_where(Expr::col(Bangumi::MikanBangumiId).is_not_null())
            .to_owned(),
        )
        .await?;

      manager
        .alter_table(
          Table::alter()
            .table(Bangumi::Table)
            .modify_column(enumeration(Bangumi::BangumiType, BangumiTypeEnum, BangumiType::iden_values()))
            .to_owned(),
        )
        .await?;
    }

    {
      create_postgres_enum_for_active_enum!(manager, EpisodeTypeEnum, EpisodeType::Mikan).await?;

      manager
        .alter_table(
          Table::alter()
            .table(Episodes::Table)
            .add_column_if_not_exists(enumeration_null(Episodes::EpisodeType, EpisodeTypeEnum, EpisodeType::enum_type_name()))
            .add_column_if_not_exists(text_null(Episodes::EnclosureMagnetLink))
            .add_column_if_not_exists(text_null(Episodes::EnclosureTorrentLink))
            .add_column_if_not_exists(timestamp_with_time_zone_null(Episodes::EnclosurePubDate))
            .add_column_if_not_exists(big_integer_null(Episodes::EnclosureContentLength))
            .to_owned(),
        )
        .await?;

      db.execute_unprepared(&format!(
        r#"ALTER TABLE {episodes} DROP COLUMN IF EXISTS {save_path}"#,
        episodes = Episodes::Table.to_string(),
        save_path = Episodes::SavePath.to_string(),
      ))
      .await?;

      manager
        .exec_stmt(
          UpdateStatement::new()
            .table(Episodes::Table)
            .value(Episodes::EpisodeType, EpisodeType::Mikan.as_enum(EpisodeTypeEnum))
            .and_where(Expr::col(Episodes::EpisodeType).is_null())
            .and_where(Expr::col(Episodes::MikanEpisodeId).is_not_null())
            .to_owned(),
        )
        .await?;

      manager
        .alter_table(
          Table::alter()
            .table(Episodes::Table)
            .modify_column(enumeration(Episodes::EpisodeType, EpisodeTypeEnum, EpisodeType::enum_type_name()))
            .to_owned(),
        )
        .await?;
    }

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Bangumi::Table)
          .add_column_if_not_exists(text_null(Bangumi::SavePath))
          .drop_column(Bangumi::BangumiType)
          .to_owned(),
      )
      .await?;

    manager.drop_postgres_enum_for_active_enum(BangumiTypeEnum).await?;

    manager
      .alter_table(
        Table::alter()
          .table(Episodes::Table)
          .add_column_if_not_exists(text_null(Episodes::SavePath))
          .drop_column(Episodes::EpisodeType)
          .drop_column(Episodes::EnclosureMagnetLink)
          .drop_column(Episodes::EnclosureTorrentLink)
          .drop_column(Episodes::EnclosurePubDate)
          .drop_column(Episodes::EnclosureContentLength)
          .to_owned(),
      )
      .await?;

    manager.drop_postgres_enum_for_active_enum(EpisodeTypeEnum).await?;

    Ok(())
  }
}
