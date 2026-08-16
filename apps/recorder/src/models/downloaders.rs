use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, DeriveDisplay, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "downloader_category")]
#[serde(rename_all = "snake_case")]
pub enum DownloaderCategory {
  #[sea_orm(string_value = "qbittorrent")]
  QBittorrent,
  #[sea_orm(string_value = "dandanplay")]
  Dandanplay,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "downloaders")]
pub struct Model {
  #[sea_orm(default_expr = "Expr::current_timestamp()")]
  pub created_at: DateTimeUtc,
  #[sea_orm(default_expr = "Expr::current_timestamp()")]
  pub updated_at: DateTimeUtc,
  #[sea_orm(primary_key)]
  pub id: i32,
  pub category: DownloaderCategory,
  pub endpoint: String,
  pub password: String,
  pub username: String,
  pub subscriber_id: i32,
  pub save_path: String,
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
  #[sea_orm(has_many = "super::downloads::Entity")]
  Download,
}

impl Related<super::subscribers::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Subscriber.def()
  }
}

impl Related<super::downloads::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Download.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
  #[sea_orm(entity = "super::subscribers::Entity")]
  Subscriber,
  #[sea_orm(entity = "super::downloads::Entity")]
  Download,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl Model {
  pub fn get_endpoint(&self) -> String {
    self.endpoint.clone()
  }

  pub fn endpoint_url(&self) -> Result<Url, url::ParseError> {
    let url = Url::parse(&self.endpoint)?;
    Ok(url)
  }
}
