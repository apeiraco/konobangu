use async_trait::async_trait;
use sea_orm::{ConnectionTrait, EntityTrait, Set, TransactionTrait, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
  app::AppContextTrait,
  errors::app_error::{RecorderError, RecorderResult},
};

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, DeriveDisplay, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "auth_type")]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
  #[sea_orm(string_value = "basic")]
  Basic,
  #[sea_orm(string_value = "oidc")]
  Oidc,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "auth", schema_name = "auth_identity")]
pub struct Model {
  #[sea_orm(default_expr = "Expr::current_timestamp()")]
  pub created_at: DateTimeUtc,
  #[sea_orm(default_expr = "Expr::current_timestamp()")]
  pub updated_at: DateTimeUtc,
  #[sea_orm(primary_key)]
  pub id: i32,
  pub pid: String,
  pub issuer: Option<String>,
  pub subscriber_id: i32,
  pub auth_type: AuthType,
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
  SubscriberId,
}

impl Related<super::subscribers::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::SubscriberId.def()
  }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl Model {
  pub async fn find_by_pid(ctx: &dyn AppContextTrait, pid: &str) -> RecorderResult<Self> {
    Entity::find()
      .filter(Column::Pid.eq(pid))
      .filter(Column::AuthType.eq(AuthType::Basic))
      .one(ctx.auth().identity_db())
      .await?
      .ok_or_else(|| RecorderError::from_entity_not_found_detail::<Entity, _>("Basic identity not found"))
  }

  pub async fn find_or_create_oidc(db: &sea_orm::DatabaseConnection, issuer: &str, subject: &str) -> Result<Self, sea_orm::DbErr> {
    let transaction = db.begin().await?;
    // Serialize only identical identity keys. A hash collision merely
    // serializes unrelated logins.
    transaction
      .query_one_raw(sea_orm::Statement::from_sql_and_values(
        sea_orm::DbBackend::Postgres,
        "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
        [serde_json::json!([issuer, subject]).to_string().into()],
      ))
      .await?;
    if let Some(identity) = Entity::find()
      .filter(Column::AuthType.eq(AuthType::Oidc))
      .filter(Column::Issuer.eq(issuer))
      .filter(Column::Pid.eq(subject))
      .one(&transaction)
      .await?
    {
      transaction.commit().await?;
      return Ok(identity);
    }
    let subscriber_id: i32 = transaction
      .query_one_raw(sea_orm::Statement::from_sql_and_values(
        sea_orm::DbBackend::Postgres,
        "INSERT INTO public.subscribers (display_name) VALUES ($1) RETURNING id",
        [subject.into()],
      ))
      .await?
      .ok_or_else(|| sea_orm::DbErr::RecordNotInserted)?
      .try_get_by_index(0)?;
    let identity = ActiveModel {
      pid: Set(subject.to_owned()),
      issuer: Set(Some(issuer.to_owned())),
      auth_type: Set(AuthType::Oidc),
      subscriber_id: Set(subscriber_id),
      ..Default::default()
    }
    .insert(&transaction)
    .await?;
    transaction.commit().await?;
    Ok(identity)
  }
}
