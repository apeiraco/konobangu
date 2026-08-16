use async_trait::async_trait;
use sea_orm::{ActiveValue, entity::prelude::*};

pub use crate::task::{SystemTask, SystemTaskInput, SystemTaskType, SystemTaskTypeEnum, SystemTaskTypeVariant, SystemTaskTypeVariantIter};

#[derive(Clone, Debug, PartialEq, Eq, DeriveActiveEnum, EnumIter, DeriveDisplay)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum SystemTaskStatus {
  #[sea_orm(string_value = "Pending")]
  Pending,
  #[sea_orm(string_value = "Scheduled")]
  Scheduled,
  #[sea_orm(string_value = "Running")]
  Running,
  #[sea_orm(string_value = "Done")]
  Done,
  #[sea_orm(string_value = "Failed")]
  Failed,
  #[sea_orm(string_value = "Killed")]
  Killed,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "system_tasks")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: String,
  pub subscriber_id: Option<i32>,
  pub cron_id: Option<i32>,
  pub job: SystemTask,
  pub task_type: SystemTaskType,
  pub status: SystemTaskStatus,
  pub attempts: i32,
  pub max_attempts: i32,
  pub run_at: DateTimeUtc,
  pub last_error: Option<String>,
  pub generation: i64,
  pub cancel_requested_at: Option<DateTimeUtc>,
  pub done_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::subscribers::Entity",
    from = "Column::SubscriberId",
    to = "super::subscribers::Column::Id",
    on_update = "Cascade",
    on_delete = "Restrict"
  )]
  Subscriber,
  #[sea_orm(
    belongs_to = "super::cron::Entity",
    from = "Column::CronId",
    to = "super::cron::Column::Id",
    on_update = "Cascade",
    on_delete = "Restrict"
  )]
  Cron,
}

impl Related<super::subscribers::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Subscriber.def()
  }
}

impl Related<super::cron::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Cron.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
  #[sea_orm(entity = "super::subscribers::Entity")]
  Subscriber,
  #[sea_orm(entity = "super::cron::Entity")]
  Cron,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
  async fn before_save<C>(mut self, _db: &C, _insert: bool) -> Result<Self, DbErr>
  where
    C: ConnectionTrait,
  {
    if let ActiveValue::Set(Some(..)) = self.subscriber_id {
      return Err(DbErr::Custom("SystemTask can not be created by subscribers now".to_string()));
    }
    Ok(self)
  }
}
