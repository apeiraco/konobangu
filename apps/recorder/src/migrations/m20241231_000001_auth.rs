use async_trait::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::defs::{Auth, table_auto_z};
use crate::{
    migrations::defs::{CustomSchemaManagerExt, GeneralIds, Subscribers},
    models::{
        auth::{AuthType, AuthTypeEnum},
        subscribers::SEED_SUBSCRIBER,
    },
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_postgres_enum_for_active_enum!(
            manager,
            AuthTypeEnum,
            AuthType::Basic,
            AuthType::Oidc
        )
        .await?;

        manager
            .create_table(
                table_auto_z(Auth::Table)
                    .col(pk_auto(Auth::Id))
                    .col(text(Auth::Pid))
                    .col(enumeration(
                        Auth::AuthType,
                        AuthTypeEnum,
                        AuthType::iden_values(),
                    ))
                    .col(integer(Auth::SubscriberId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_auth_subscriber_id")
                            .from_tbl(Auth::Table)
                            .from_col(Auth::SubscriberId)
                            .to_tbl(Subscribers::Table)
                            .to_col(Subscribers::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_auth_pid_auth_type")
                    .unique()
                    .table(Auth::Table)
                    .col(Auth::Pid)
                    .col(Auth::AuthType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_postgres_auto_update_ts_trigger_for_col(Auth::Table, GeneralIds::UpdatedAt)
            .await?;

        let seed_subscriber_id = manager
            .get_connection()
            .query_one_raw(
                manager.get_database_backend().build(
                    Query::select()
                        .column(Subscribers::Id)
                        .from(Subscribers::Table)
                        .limit(1),
                ),
            )
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(String::from("seed subscriber not found")))?
            .try_get_by_index::<i32>(0)?;

        manager
            .exec_stmt(
                Query::insert()
                    .into_table(Auth::Table)
                    .columns([Auth::Pid, Auth::AuthType, Auth::SubscriberId])
                    .values_panic([
                        SEED_SUBSCRIBER.into(),
                        SimpleExpr::from(AuthType::Basic).as_enum(AuthTypeEnum),
                        seed_subscriber_id.into(),
                    ])
                    .on_conflict(
                        OnConflict::columns([Auth::Pid, Auth::AuthType])
                            .do_nothing()
                            .to_owned(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_postgres_auto_update_ts_trigger_for_col(Auth::Table, GeneralIds::UpdatedAt)
            .await?;

        manager
            .drop_table(Table::drop().if_exists().table(Auth::Table).to_owned())
            .await?;

        manager
            .drop_postgres_enum_for_active_enum(AuthTypeEnum)
            .await?;

        Ok(())
    }
}
