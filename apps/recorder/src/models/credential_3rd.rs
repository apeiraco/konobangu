use async_trait::async_trait;
use sea_orm::{ActiveValue, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
  app::AppContextTrait,
  crypto::UserPassCredential,
  errors::{RecorderError, RecorderResult},
};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, DeriveDisplay, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "credential_3rd_type")]
#[serde(rename_all = "snake_case")]
pub enum Credential3rdType {
  #[sea_orm(string_value = "mikan")]
  Mikan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "credential3rd")]
pub struct Model {
  #[sea_orm(default_expr = "Expr::current_timestamp()")]
  pub created_at: DateTimeUtc,
  #[sea_orm(default_expr = "Expr::current_timestamp()")]
  pub updated_at: DateTimeUtc,
  #[sea_orm(primary_key)]
  pub id: i32,
  pub subscriber_id: i32,
  pub credential_type: Credential3rdType,
  pub cookies: Option<String>,
  pub username: Option<String>,
  pub password: Option<String>,
  pub user_agent: Option<String>,
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
  #[sea_orm(has_many = "super::subscriptions::Entity")]
  Subscription,
}

impl Related<super::subscribers::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Subscriber.def()
  }
}

impl Related<super::subscriptions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Subscription.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
  #[sea_orm(entity = "super::subscribers::Entity")]
  Subscriber,
  #[sea_orm(entity = "super::subscriptions::Entity")]
  Subscription,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
  pub async fn try_encrypt(mut self, ctx: &dyn AppContextTrait) -> RecorderResult<Self> {
    let crypto = ctx.crypto();

    if let ActiveValue::Set(Some(username)) = self.username {
      let username_enc = crypto.encrypt_string(username)?;
      self.username = ActiveValue::Set(Some(username_enc));
    }

    if let ActiveValue::Set(Some(password)) = self.password {
      let password_enc = crypto.encrypt_string(password)?;
      self.password = ActiveValue::Set(Some(password_enc));
    }

    if let ActiveValue::Set(Some(cookies)) = self.cookies {
      let cookies_enc = crypto.encrypt_string(cookies)?;
      self.cookies = ActiveValue::Set(Some(cookies_enc));
    }

    Ok(self)
  }
}

impl Model {
  pub async fn find_by_id_and_subscriber_id(ctx: &dyn AppContextTrait, id: i32, subscriber_id: i32) -> RecorderResult<Option<Self>> {
    let transaction = crate::database::operation::begin_task_transaction(ctx.db().as_ref(), subscriber_id).await?;
    let db = &transaction;
    let credential = Entity::find()
      .filter(Column::Id.eq(id))
      .filter(Column::SubscriberId.eq(subscriber_id))
      .one(db)
      .await?;

    transaction.rollback().await?;
    Ok(credential)
  }

  pub fn try_into_userpass_credential(self, ctx: &dyn AppContextTrait) -> RecorderResult<UserPassCredential> {
    let crypto = ctx.crypto();
    let username_enc = self.username.ok_or_else(|| RecorderError::Credential3rdError {
      message: "UserPassCredential username is required".to_string(),
      source: None.into(),
    })?;

    let username: String = crypto.decrypt_string(&username_enc)?;

    let password_enc = self.password.ok_or_else(|| RecorderError::Credential3rdError {
      message: "UserPassCredential password is required".to_string(),
      source: None.into(),
    })?;

    let password: String = crypto.decrypt_string(&password_enc)?;

    let cookies: Option<String> = if let Some(cookies_enc) = self.cookies {
      let cookies = crypto.decrypt_string(&cookies_enc)?;
      Some(cookies)
    } else {
      None
    };

    Ok(UserPassCredential {
      username,
      password,
      cookies,
      user_agent: self.user_agent,
    })
  }

  pub async fn check_remote_availability(self, ctx: &dyn AppContextTrait) -> RecorderResult<(bool, Option<String>)> {
    match self.credential_type {
      Credential3rdType::Mikan => {
        let mikan_client = {
          let userpass_credential: UserPassCredential = self.try_into_userpass_credential(ctx)?;
          ctx.mikan().fork_with_userpass_credential(userpass_credential).await?
        };
        let mut has_login = mikan_client.has_login().await?;
        if !has_login {
          mikan_client.login().await?;
          has_login = true;
        }
        Ok((has_login, mikan_client.credential_cookies()?))
      }
    }
  }
}
