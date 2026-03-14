use async_trait::async_trait;
use sea_orm::{EntityTrait, Set, TransactionTrait, prelude::*, sea_query::ExprTrait};
use serde::{Deserialize, Serialize};

use super::subscribers::{self, SEED_SUBSCRIBER};
use crate::{
    app::AppContextTrait,
    errors::app_error::{RecorderError, RecorderResult},
};

#[derive(
    Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, DeriveDisplay, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "auth_type")]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    #[sea_orm(string_value = "basic")]
    Basic,
    #[sea_orm(string_value = "oidc")]
    Oidc,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "auth")]
pub struct Model {
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeUtc,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: String,
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
        let db = ctx.db();
        let subscriber_auth = Entity::find()
            .filter(Column::Pid.eq(pid))
            .one(db)
            .await?
            .ok_or_else(|| {
                RecorderError::from_entity_not_found_detail::<Entity, _>(format!(
                    "pid {pid} not found"
                ))
            })?;
        Ok(subscriber_auth)
    }

    pub async fn create_from_oidc(ctx: &dyn AppContextTrait, sub: String) -> RecorderResult<Self> {
        let db = ctx.db();

        let txn = db.begin().await?;

        let subscriber_id = if let Some(seed_subscriber_id) = Entity::find()
            .filter(
                Column::AuthType
                    .eq(AuthType::Basic)
                    .and(Column::Pid.eq(SEED_SUBSCRIBER)),
            )
            .one(&txn)
            .await?
            .map(|m| m.subscriber_id)
        {
            seed_subscriber_id
        } else {
            let new_subscriber = subscribers::ActiveModel {
                ..Default::default()
            };
            let new_subscriber: subscribers::Model = new_subscriber.save(&txn).await?.try_into()?;

            new_subscriber.id
        };

        let new_item = ActiveModel {
            pid: Set(sub),
            auth_type: Set(AuthType::Oidc),
            subscriber_id: Set(subscriber_id),
            ..Default::default()
        };

        let new_item: Model = new_item.insert(&txn).await?;

        txn.commit().await?;

        Ok(new_item)
    }
}
