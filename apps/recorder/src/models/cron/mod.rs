mod core;

pub use core::{
  CHECK_AND_TRIGGER_DUE_CRONS_FUNCTION_NAME, CRON_DUE_DEBUG_EVENT, CRON_DUE_EVENT, NOTIFY_DUE_CRON_WHEN_MUTATING_FUNCTION_NAME,
  NOTIFY_DUE_CRON_WHEN_MUTATING_TRIGGER_NAME, SETUP_CRON_EXTRA_FOREIGN_KEYS_FUNCTION_NAME, SETUP_CRON_EXTRA_FOREIGN_KEYS_TRIGGER_NAME,
};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Offset, TimeZone, Utc};
use chrono_tz::Tz;
use croner::parser::{CronParser, Seconds};
use sea_orm::{
  ActiveValue::{self, Set},
  DeriveActiveEnum, DeriveDisplay, DeriveEntityModel, EnumIter, QueryOrder, QuerySelect, TransactionTrait,
  entity::prelude::*,
  sea_query::{LockBehavior, LockType},
};
use serde::{Deserialize, Serialize};

use crate::{
  errors::RecorderResult,
  models::{subscriber_tasks, system_tasks},
  task::{SubscriberTaskTrait, SystemTaskTrait},
};

#[derive(Debug, Clone, PartialEq, Eq, DeriveActiveEnum, EnumIter, DeriveDisplay, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "cron_status")]
#[serde(rename_all = "snake_case")]
pub enum CronStatus {
  #[sea_orm(string_value = "pending")]
  Pending,
  #[sea_orm(string_value = "running")]
  Running,
  #[sea_orm(string_value = "completed")]
  Completed,
  #[sea_orm(string_value = "failed")]
  Failed,
  #[sea_orm(string_value = "disabled")]
  Disabled,
}

#[derive(Debug, Clone, DeriveEntityModel, PartialEq, Serialize, Deserialize)]
#[sea_orm(table_name = "cron")]
pub struct Model {
  #[sea_orm(default_expr = "Expr::current_timestamp()")]
  pub created_at: DateTimeUtc,
  #[sea_orm(default_expr = "Expr::current_timestamp()")]
  pub updated_at: DateTimeUtc,
  #[sea_orm(primary_key)]
  pub id: i32,
  pub subscriber_id: Option<i32>,
  pub subscription_id: Option<i32>,
  pub cron_expr: String,
  pub cron_timezone: String,
  pub next_run: Option<DateTimeUtc>,
  pub last_run: Option<DateTimeUtc>,
  pub last_error: Option<String>,
  pub locked_by: Option<String>,
  pub locked_at: Option<DateTimeUtc>,
  // default_expr = "5000"
  pub timeout_ms: Option<i32>,
  #[sea_orm(default_expr = "0")]
  pub attempts: i32,
  #[sea_orm(default_expr = "1")]
  pub max_attempts: i32,
  #[sea_orm(default_expr = "0")]
  pub priority: i32,
  pub status: CronStatus,
  #[sea_orm(default_expr = "true")]
  pub enabled: bool,
  pub subscriber_task_cron: Option<subscriber_tasks::SubscriberTask>,
  pub system_task_cron: Option<system_tasks::SystemTask>,
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
    belongs_to = "super::subscriptions::Entity",
    from = "Column::SubscriptionId",
    to = "super::subscriptions::Column::Id",
    on_update = "Cascade",
    on_delete = "Restrict"
  )]
  Subscription,
  #[sea_orm(has_many = "super::subscriber_tasks::Entity")]
  SubscriberTask,
  #[sea_orm(has_many = "super::system_tasks::Entity")]
  SystemTask,
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

impl Related<super::subscriber_tasks::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::SubscriberTask.def()
  }
}

impl Related<super::system_tasks::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::SystemTask.def()
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
  #[sea_orm(entity = "super::subscribers::Entity")]
  Subscriber,
  #[sea_orm(entity = "super::subscriptions::Entity")]
  Subscription,
  #[sea_orm(entity = "super::subscriber_tasks::Entity")]
  SubscriberTask,
  #[sea_orm(entity = "super::system_tasks::Entity")]
  SystemTask,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
  async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
  where
    C: ConnectionTrait,
  {
    match (&self.cron_expr as &ActiveValue<String>, &self.cron_timezone as &ActiveValue<String>) {
      (ActiveValue::Set(cron_expr), ActiveValue::Set(timezone)) => {
        if matches!(&self.next_run, ActiveValue::NotSet | ActiveValue::Unchanged(_)) {
          let next_run = Model::calculate_next_run(cron_expr, timezone).map_err(|e| DbErr::Custom(e.to_string()))?;
          self.next_run = Set(Some(next_run));
        }
      }
      (ActiveValue::Unchanged(_) | ActiveValue::NotSet, ActiveValue::Unchanged(_) | ActiveValue::NotSet) => {}
      (_, _) => {
        if matches!(self.next_run, ActiveValue::NotSet | ActiveValue::Unchanged(_)) {
          return Err(DbErr::Custom(
            "Cron expr and timezone must be insert or update at same time when next run is not set".to_string(),
          ));
        }
      }
    };
    if let ActiveValue::Set(Some(subscriber_id)) = self.subscriber_id
      && let ActiveValue::Set(Some(ref subscriber_task)) = self.subscriber_task_cron
      && subscriber_task.get_subscriber_id() != subscriber_id
    {
      return Err(DbErr::Custom(
        "Cron subscriber_id does not match subscriber_task_cron.subscriber_id".to_string(),
      ));
    }
    if let ActiveValue::Set(Some(subscriber_id)) = self.subscriber_id
      && let ActiveValue::Set(Some(ref system_task)) = self.system_task_cron
      && system_task.get_subscriber_id() != Some(subscriber_id)
    {
      return Err(DbErr::Custom("Cron subscriber_id does not match system_task_cron.subscriber_id".to_string()));
    }
    if let ActiveValue::Set(enabled) = self.enabled
      && !insert
    {
      if enabled {
        self.status = Set(CronStatus::Pending)
      } else {
        self.status = Set(CronStatus::Disabled)
      }
    }

    Ok(self)
  }
}

impl Model {
  /// Polling is the correctness path; the due-row lock, task/outbox insert and
  /// schedule advance belong to one transaction shared by competing schedulers.
  pub async fn dispatch_due(db: &sea_orm::DatabaseConnection, now: DateTime<Utc>) -> RecorderResult<usize> {
    let mut advanced = 0;
    for _ in 0..100 {
      let transaction = db.begin().await?;
      let due = Entity::find()
        .filter(Column::Enabled.eq(true))
        .filter(Column::NextRun.lte(now))
        .order_by_asc(Column::NextRun)
        .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
        .one(&transaction)
        .await?;
      let Some(due) = due else {
        transaction.rollback().await?;
        break;
      };
      if let Some(subscription) = due.subscription_id
        && !crate::task::operation::try_lock_subscription_intent(&transaction, i64::from(subscription)).await?
      {
        transaction.rollback().await?;
        break;
      }
      let scheduled = due.next_run.ok_or_else(|| DbErr::Custom("Due cron has no occurrence".into()))?;
      let active = transaction
        .query_one_raw(sea_orm::Statement::from_sql_and_values(
          sea_orm::DbBackend::Postgres,
          "SELECT id FROM task_runs WHERE cron_id=$1 AND status IN ('Pending','Scheduled','Running') AND cancel_requested_at IS NULL AND archived_at IS NULL \
           LIMIT 1",
          [due.id.into()],
        ))
        .await?
        .is_some();
      if !active {
        if let Some(mut task) = due.subscriber_task_cron.clone() {
          task.set_subscriber_id(due.subscriber_id.ok_or_else(|| DbErr::Custom("Subscriber cron has no trusted owner".into()))?);
          task.set_cron_id(Some(due.id));
          crate::task::operation::enqueue_cron(&transaction, "subscriber_task", &task, scheduled).await?;
        } else if let Some(mut task) = due.system_task_cron.clone() {
          task.set_subscriber_id(due.subscriber_id);
          task.set_cron_id(Some(due.id));
          crate::task::operation::enqueue_cron(&transaction, "system_task", &task, scheduled).await?;
        } else {
          return Err(DbErr::Custom("Cron has no valid task payload".into()).into());
        }
      }
      ActiveModel {
        id: Set(due.id),
        next_run: Set(Some(Self::calculate_next_run_after(&due.cron_expr, &due.cron_timezone, now)?)),
        last_run: Set(Some(scheduled)),
        status: Set(CronStatus::Pending),
        locked_by: Set(None),
        locked_at: Set(None),
        attempts: Set(0),
        last_error: Set(active.then(|| "Occurrence skipped: prior business run is still active".into())),
        ..Default::default()
      }
      .update(&transaction)
      .await?;
      transaction.commit().await?;
      advanced += 1;
    }
    Ok(advanced)
  }

  pub fn calculate_next_run(cron_expr: &str, timezone: &str) -> RecorderResult<DateTime<Utc>> {
    Self::calculate_next_run_after(cron_expr, timezone, Utc::now())
  }

  pub fn calculate_next_run_after(cron_expr: &str, timezone: &str, after: DateTime<Utc>) -> RecorderResult<DateTime<Utc>> {
    let user_tz = timezone.parse::<Tz>()?;
    let cron = CronParser::builder().seconds(Seconds::Optional).build().parse(cron_expr)?;
    let mut next = cron.find_next_occurrence(&after.with_timezone(&user_tz), false)?.with_timezone(&Utc);
    // Croner may shift a nonexistent wall time to the transition boundary.
    // A shifted time is not an occurrence of this expression.
    while !cron.is_time_matching(&next.with_timezone(&user_tz))? {
      next = cron.find_next_occurrence(&next.with_timezone(&user_tz), false)?.with_timezone(&Utc);
    }
    // Croner chooses one side of an ambiguous wall time. Enumerate both legs
    // of a nearby backward transition using its same wall-clock matcher.
    let left = after - Duration::days(1);
    let right = after + Duration::days(1);
    let old = left.with_timezone(&user_tz).offset().fix().local_minus_utc();
    let new = right.with_timezone(&user_tz).offset().fix().local_minus_utc();
    if old > new {
      let mut lo = left.timestamp();
      let mut hi = right.timestamp();
      while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if Utc.timestamp_opt(mid, 0).unwrap().with_timezone(&user_tz).offset().fix().local_minus_utc() == old {
          lo = mid;
        } else {
          hi = mid;
        }
      }
      let transition = Utc.timestamp_opt(hi, 0).unwrap();
      let repeated = Duration::seconds(i64::from(old - new));
      for (offset, start, end) in [(old, transition - repeated, transition), (new, transition, transition + repeated)] {
        let lower = (after + Duration::seconds(1)).max(start);
        if lower >= end {
          continue;
        }
        let wall = lower + Duration::seconds(i64::from(offset));
        let candidate = cron.find_next_occurrence(&wall, true)? - Duration::seconds(i64::from(offset));
        if candidate > after && candidate < end && candidate < next {
          next = candidate;
        }
      }
    }
    Ok(next)
  }
}
