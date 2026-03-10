mod core;

pub use core::{
    CHECK_AND_TRIGGER_DUE_CRONS_FUNCTION_NAME, CRON_DUE_DEBUG_EVENT, CRON_DUE_EVENT,
    NOTIFY_DUE_CRON_WHEN_MUTATING_FUNCTION_NAME, NOTIFY_DUE_CRON_WHEN_MUTATING_TRIGGER_NAME,
    SETUP_CRON_EXTRA_FOREIGN_KEYS_FUNCTION_NAME, SETUP_CRON_EXTRA_FOREIGN_KEYS_TRIGGER_NAME,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use croner::parser::{CronParser, Seconds};
use sea_orm::{
    ActiveValue::{self, Set},
    Condition, DeriveActiveEnum, DeriveDisplay, DeriveEntityModel, EnumIter, QuerySelect,
    Statement, TransactionTrait,
    entity::prelude::*,
    sea_query::{ExprTrait, LockBehavior, LockType},
    sqlx::postgres::PgNotification,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    models::{subscriber_tasks, system_tasks},
    task::{SubscriberTaskTrait, SystemTaskTrait},
};

#[derive(
    Debug, Clone, PartialEq, Eq, DeriveActiveEnum, EnumIter, DeriveDisplay, Serialize, Deserialize,
)]
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
        match (
            &self.cron_expr as &ActiveValue<String>,
            &self.cron_timezone as &ActiveValue<String>,
        ) {
            (ActiveValue::Set(cron_expr), ActiveValue::Set(timezone)) => {
                if matches!(
                    &self.next_run,
                    ActiveValue::NotSet | ActiveValue::Unchanged(_)
                ) {
                    let next_run = Model::calculate_next_run(cron_expr, timezone)
                        .map_err(|e| DbErr::Custom(e.to_string()))?;
                    self.next_run = Set(Some(next_run));
                }
            }
            (
                ActiveValue::Unchanged(_) | ActiveValue::NotSet,
                ActiveValue::Unchanged(_) | ActiveValue::NotSet,
            ) => {}
            (_, _) => {
                if matches!(
                    self.next_run,
                    ActiveValue::NotSet | ActiveValue::Unchanged(_)
                ) {
                    return Err(DbErr::Custom(
                        "Cron expr and timezone must be insert or update at same time when next \
                         run is not set"
                            .to_string(),
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
            return Err(DbErr::Custom(
                "Cron subscriber_id does not match system_task_cron.subscriber_id".to_string(),
            ));
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
    pub async fn handle_cron_notification(
        ctx: &dyn AppContextTrait,
        notification: PgNotification,
        worker_id: &str,
        retry_duration: chrono::Duration,
    ) -> RecorderResult<()> {
        let payload: Self = serde_json::from_str(notification.payload())?;
        let cron_id = payload.id;

        tracing::debug!("Cron notification received for cron {cron_id} and worker {worker_id}");

        match Self::try_acquire_lock_with_cron_id(ctx, cron_id, worker_id).await? {
            Some(cron) => match cron.exec_cron(ctx).await {
                Ok(()) => {
                    tracing::debug!("Cron {cron_id} executed successfully");
                    cron.mark_cron_completed(ctx).await?;
                }
                Err(e) => {
                    tracing::error!("Error executing cron {cron_id}: {e}");
                    cron.mark_cron_failed(ctx, &e.to_string(), retry_duration)
                        .await?;
                }
            },
            None => {
                tracing::debug!(
                    "Cron lock not acquired for cron {cron_id} and worker {worker_id}, skipping..."
                );
            }
        }

        Ok(())
    }

    async fn try_acquire_lock_with_cron_id(
        ctx: &dyn AppContextTrait,
        cron_id: i32,
        worker_id: &str,
    ) -> RecorderResult<Option<Self>> {
        let db = ctx.db();
        let txn = db.begin().await?;

        let cron = Entity::find_by_id(cron_id)
            .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
            .one(&txn)
            .await?;

        if let Some(cron) = cron {
            if cron.enabled
                && cron.attempts < cron.max_attempts
                && cron.status == CronStatus::Pending
                && (cron.locked_at.is_none_or(|locked_at| {
                    cron.timeout_ms.is_some_and(|cron_timeout_ms| {
                        locked_at + chrono::Duration::milliseconds(cron_timeout_ms as i64)
                            <= Utc::now()
                    })
                }))
                && cron.next_run.is_some_and(|next_run| next_run <= Utc::now())
            {
                let cron_active_model = ActiveModel {
                    id: Set(cron.id),
                    locked_by: Set(Some(worker_id.to_string())),
                    locked_at: Set(Some(Utc::now())),
                    status: Set(CronStatus::Running),
                    attempts: Set(cron.attempts + 1),
                    ..Default::default()
                };
                let cron_model = cron_active_model.update(&txn).await?;
                txn.commit().await?;
                return Ok(Some(cron_model));
            }
            txn.commit().await?;
            return Ok(Some(cron));
        }
        txn.rollback().await?;
        Ok(None)
    }

    async fn exec_cron(&self, ctx: &dyn AppContextTrait) -> RecorderResult<()> {
        if let Some(subscriber_task) = self.subscriber_task_cron.as_ref() {
            let task_service = ctx.task();
            let mut new_subscriber_task = subscriber_task.clone();
            new_subscriber_task.set_cron_id(Some(self.id));
            task_service
                .add_subscriber_task(new_subscriber_task)
                .await?;
        } else if let Some(system_task) = self.system_task_cron.as_ref() {
            let task_service = ctx.task();
            let mut new_system_task = system_task.clone();
            new_system_task.set_cron_id(Some(self.id));
            task_service.add_system_task(new_system_task).await?;
        } else {
            unimplemented!("Cron without unknown task is not supported now");
        }

        Ok(())
    }

    async fn mark_cron_completed(&self, ctx: &dyn AppContextTrait) -> RecorderResult<()> {
        let db = ctx.db();

        let next_run = Self::calculate_next_run(&self.cron_expr, &self.cron_timezone)?;

        ActiveModel {
            id: Set(self.id),
            next_run: Set(Some(next_run)),
            last_run: Set(Some(Utc::now())),
            status: Set(CronStatus::Pending),
            locked_by: Set(None),
            locked_at: Set(None),
            attempts: Set(0),
            last_error: Set(None),
            ..Default::default()
        }
        .update(db)
        .await?;

        Ok(())
    }

    async fn mark_cron_failed(
        &self,
        ctx: &dyn AppContextTrait,
        error: &str,
        retry_duration: chrono::Duration,
    ) -> RecorderResult<()> {
        let db = ctx.db();

        let should_retry = self.attempts < self.max_attempts;

        let status = if should_retry {
            CronStatus::Pending
        } else {
            CronStatus::Failed
        };

        let next_run = if should_retry {
            Some(Utc::now() + retry_duration)
        } else {
            Some(Self::calculate_next_run(
                &self.cron_expr,
                &self.cron_timezone,
            )?)
        };

        ActiveModel {
            id: Set(self.id),
            next_run: Set(next_run),
            status: Set(status),
            locked_by: Set(None),
            locked_at: Set(None),
            last_run: Set(Some(Utc::now())),
            last_error: Set(Some(error.to_string())),
            attempts: Set(if should_retry { self.attempts + 1 } else { 0 }),
            ..Default::default()
        }
        .update(db)
        .await?;

        Ok(())
    }

    pub async fn check_and_trigger_due_crons(ctx: &dyn AppContextTrait) -> RecorderResult<()> {
        let db = ctx.db();

        db.execute(Statement::from_string(
            db.get_database_backend(),
            format!("SELECT {CHECK_AND_TRIGGER_DUE_CRONS_FUNCTION_NAME}()"),
        ))
        .await?;

        Ok(())
    }

    pub async fn check_and_cleanup_expired_cron_locks(
        ctx: &dyn AppContextTrait,
        retry_duration: chrono::Duration,
    ) -> RecorderResult<()> {
        let db = ctx.db();

        let condition = Condition::all()
            .add(Column::Status.eq(CronStatus::Running))
            .add(Column::LastRun.is_not_null())
            .add(Column::TimeoutMs.is_not_null())
            .add(
                Expr::col(Column::LastRun)
                    .add(Expr::col(Column::TimeoutMs).mul(Expr::cust("INTERVAL '1 millisecond'")))
                    .lte(Expr::current_timestamp()),
            );

        let cron_ids = Entity::find()
            .select_only()
            .column(Column::Id)
            .filter(condition.clone())
            .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
            .into_tuple::<i32>()
            .all(db)
            .await?;

        for cron_id in cron_ids {
            let txn = db.begin().await?;
            let locked_cron = Entity::find_by_id(cron_id)
                .filter(condition.clone())
                .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
                .one(&txn)
                .await?;

            if let Some(locked_cron) = locked_cron {
                locked_cron
                    .mark_cron_failed(
                        ctx,
                        format!(
                            "Cron timeout of {}ms",
                            locked_cron
                                .timeout_ms
                                .as_ref()
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| "Infinite".to_string())
                        )
                        .as_str(),
                        retry_duration,
                    )
                    .await?;
                txn.commit().await?;
            } else {
                txn.rollback().await?;
            }
        }
        Ok(())
    }

    pub fn calculate_next_run(cron_expr: &str, timezone: &str) -> RecorderResult<DateTime<Utc>> {
        let user_tz = timezone.parse::<Tz>()?;

        let user_tz_now = Utc::now().with_timezone(&user_tz);

        let cron_expr = CronParser::builder().seconds(Seconds::Optional).build().parse(cron_expr)?;

        let next = cron_expr.find_next_occurrence(&user_tz_now, false)?;

        let next_utc = next.with_timezone(&Utc);

        Ok(next_utc)
    }
}
