use async_trait::async_trait;
use sea_orm_migration::{
  prelude::*,
  schema::{string_null, *},
};

use super::defs::{CustomSchemaManagerExt, GeneralIds, table_auto_z};
use crate::{
  migrations::defs::{Credential3rd, Subscribers, Subscriptions},
  models::credential_3rd::{Credential3rdType, Credential3rdTypeEnum},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    create_postgres_enum_for_active_enum!(manager, Credential3rdTypeEnum, Credential3rdType::Mikan).await?;

    manager
      .create_table(
        table_auto_z(Credential3rd::Table)
          .col(pk_auto(Credential3rd::Id))
          .col(integer(Credential3rd::SubscriberId))
          .col(enumeration(
            Credential3rd::CredentialType,
            Credential3rdTypeEnum,
            Credential3rdType::iden_values(),
          ))
          .col(string_null(Credential3rd::Cookies))
          .col(string_null(Credential3rd::Username))
          .col(string_null(Credential3rd::Password))
          .col(string_null(Credential3rd::UserAgent))
          .foreign_key(
            ForeignKey::create()
              .name("fk_credential_3rd_subscriber_id")
              .from(Credential3rd::Table, Credential3rd::SubscriberId)
              .to(Subscribers::Table, Subscribers::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .if_not_exists()
          .name("idx_credential_3rd_credential_type")
          .table(Credential3rd::Table)
          .col(Credential3rd::CredentialType)
          .to_owned(),
      )
      .await?;

    manager
      .create_postgres_auto_update_ts_trigger_for_col(Credential3rd::Table, GeneralIds::UpdatedAt)
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(Subscriptions::Table)
          .add_column_if_not_exists(integer_null(Subscriptions::CredentialId))
          .add_foreign_key(
            TableForeignKey::new()
              .name("fk_subscriptions_credential_id")
              .from_tbl(Subscriptions::Table)
              .from_col(Subscriptions::CredentialId)
              .to_tbl(Credential3rd::Table)
              .to_col(Credential3rd::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Subscriptions::Table)
          .drop_column(Subscriptions::CredentialId)
          .drop_foreign_key("fk_subscriptions_credential_id")
          .to_owned(),
      )
      .await?;

    manager
      .drop_postgres_auto_update_ts_trigger_for_col(Credential3rd::Table, GeneralIds::UpdatedAt)
      .await?;

    manager.drop_table(Table::drop().if_exists().table(Credential3rd::Table).to_owned()).await?;

    manager.drop_postgres_enum_for_active_enum(Credential3rdTypeEnum).await?;

    Ok(())
  }
}
