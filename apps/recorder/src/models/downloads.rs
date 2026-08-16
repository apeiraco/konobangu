use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, DeriveDisplay, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "download_status")]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
  #[sea_orm(string_value = "pending")]
  Pending,
  #[sea_orm(string_value = "downloading")]
  Downloading,
  #[sea_orm(string_value = "paused")]
  Paused,
  #[sea_orm(string_value = "completed")]
  Completed,
  #[sea_orm(string_value = "failed")]
  Failed,
  #[sea_orm(string_value = "deleted")]
  Deleted,
}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, DeriveDisplay, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "download_mime")]
pub enum DownloadMime {
  #[sea_orm(string_value = "application/octet-stream")]
  #[serde(rename = "application/octet-stream")]
  OctetStream,
  #[sea_orm(string_value = "application/x-bittorrent")]
  #[serde(rename = "application/x-bittorrent")]
  BitTorrent,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "downloads")]
pub struct Model {
  #[sea_orm(default_expr = "Expr::current_timestamp()")]
  pub created_at: DateTimeUtc,
  #[sea_orm(default_expr = "Expr::current_timestamp()")]
  pub updated_at: DateTimeUtc,
  #[sea_orm(primary_key)]
  pub id: i32,
  pub origin_name: String,
  pub display_name: String,
  pub downloader_id: i32,
  pub episode_id: i32,
  pub subscriber_id: i32,
  pub status: DownloadStatus,
  pub mime: DownloadMime,
  pub url: String,
  pub all_size: Option<i64>,
  pub curr_size: Option<i64>,
  pub homepage: Option<String>,
  pub save_path: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::subscribers::Entity",
    from = "Column::SubscriberId",
    to = "super::subscribers::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Subscriber,
  #[sea_orm(
    belongs_to = "super::downloaders::Entity",
    from = "Column::DownloaderId",
    to = "super::downloaders::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Downloader,
  #[sea_orm(
    belongs_to = "super::episodes::Entity",
    from = "Column::EpisodeId",
    to = "super::episodes::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Episode,
}

impl Related<super::subscribers::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Subscriber.def()
  }
}

impl Related<super::downloaders::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Downloader.def()
  }
}

impl Related<super::episodes::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Episode.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
  #[sea_orm(entity = "super::subscribers::Entity")]
  Subscriber,
  #[sea_orm(entity = "super::downloaders::Entity")]
  Downloader,
  #[sea_orm(entity = "super::episodes::Entity")]
  Episode,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {}
