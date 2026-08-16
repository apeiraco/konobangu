use std::{future::Future, sync::Arc, time::Duration};

use apalis::prelude::*;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement, TransactionTrait};
use tokio::sync::watch;
use uuid::Uuid;

use super::{
  AsyncTaskTrait, SubscriberTask, SubscriberTaskTrait, SystemTask, TaskConfig, delivery,
  execution::{BusinessRetryPolicy, CURRENT_FENCE, ExecutionError, TaskEnvelope, TaskFence},
};
use crate::{
  app::AppContextTrait,
  database::roles::TASK_CONTROL_ACCESS_ROLE,
  errors::{RecorderError, RecorderResult},
  models::cron,
};

pub struct TaskService {
  pub config: TaskConfig,
  ctx: Arc<dyn AppContextTrait>,
  queue_database: Option<DatabaseConnection>,
  shutdown: watch::Sender<bool>,
}

impl TaskService {
  pub async fn from_config_and_ctx(config: TaskConfig, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Self> {
    let queue_database = if let Some(uri) = &config.queue_database_uri {
      let mut options = sea_orm::ConnectOptions::new(uri);
      options.sqlx_logging(false);
      crate::database::roles::restrict_pool(&mut options, TASK_CONTROL_ACCESS_ROLE);
      let db = Database::connect(options)
        .await
        .map_err(|_| DbErr::Custom("Task-control database connection failed (details redacted)".into()))?;
      let validated = async {
        let row = db
          .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT rolsuper, rolbypassrls, pg_has_role(current_user, $1::text, 'USAGE') AS task_control, EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace \
             n ON n.oid=c.relnamespace WHERE n.nspname IN ('public','apalis','auth_identity','auth_session') AND \
             pg_has_role(current_user,c.relowner,'MEMBER')) AS owner, (EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE \
             n.nspname='public' AND c.relname IN \
             ('subscriptions','bangumi','episodes','subscription_bangumi','subscription_episode','downloaders','downloads','credential3rd','feeds') AND \
             has_table_privilege(current_user,c.oid,'SELECT,INSERT,UPDATE,DELETE')) OR has_schema_privilege(current_user,'auth_identity','USAGE') OR \
             has_schema_privilege(current_user,'auth_session','USAGE')) AS private_access FROM pg_roles WHERE rolname=current_user",
            [TASK_CONTROL_ACCESS_ROLE.into()],
          ))
          .await?
          .ok_or_else(|| DbErr::Custom("Task-control database role is missing".into()))?;
        if row.try_get::<bool>("", "rolsuper")?
          || row.try_get::<bool>("", "rolbypassrls")?
          || row.try_get::<bool>("", "owner")?
          || row.try_get::<bool>("", "private_access")?
          || !row.try_get::<bool>("", "task_control")?
        {
          return Err(DbErr::Custom(
            "Task-control pool requires inherited task-control access without private application access".into(),
          ));
        }
        Ok::<_, DbErr>(())
      }
      .await;
      if let Err(error) = validated {
        let _ = db.close().await;
        return Err(error.into());
      }
      Some(db)
    } else {
      None
    };
    let (shutdown, _) = watch::channel(false);
    Ok(Self {
      config,
      ctx,
      queue_database,
      shutdown,
    })
  }
  pub async fn close(&self) -> RecorderResult<()> {
    self.shutdown.send_replace(true);
    if let Some(db) = &self.queue_database {
      db.clone().close().await?;
    }
    Ok(())
  }

  pub fn queue_database(&self) -> RecorderResult<&DatabaseConnection> {
    self
      .queue_database
      .as_ref()
      .ok_or_else(|| DbErr::Custom("Task workers require scheduler.database.url with restricted task-control access".into()).into())
  }
  pub async fn add_subscriber_task(&self, task: SubscriberTask) -> RecorderResult<String> {
    let transaction = crate::database::operation::begin_task_transaction(self.ctx.db().as_ref(), task.get_subscriber_id()).await?;
    let id = super::operation::enqueue(&transaction, "subscriber_task", &task).await?;
    crate::database::operation::commit_task_transaction(transaction).await?;
    Ok(id)
  }
  pub async fn add_system_task(&self, task: SystemTask) -> RecorderResult<String> {
    let transaction = self.queue_database()?.begin().await?;
    let id = super::operation::enqueue(&transaction, "system_task", &task).await?;
    crate::database::operation::commit_task_transaction(transaction).await?;
    Ok(id)
  }
  pub async fn add_subscriber_task_cron(&self, model: cron::ActiveModel) -> RecorderResult<cron::Model> {
    Ok(model.insert(self.queue_database()?).await?)
  }
  pub async fn add_system_task_cron(&self, model: cron::ActiveModel) -> RecorderResult<cron::Model> {
    Ok(model.insert(self.queue_database()?).await?)
  }
  pub async fn run(&self) -> RecorderResult<()> {
    self.run_with_signal(None::<fn() -> std::future::Ready<()>>).await
  }

  pub async fn run_with_signal<F, Fut>(&self, signal: Option<F>) -> RecorderResult<()>
  where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send,
  {
    let database = self.queue_database()?;
    self.shutdown.send_replace(false);
    let shutdown = &self.shutdown;
    let receiver = shutdown.subscribe();
    let worker = WorkerBuilder::new(format!("business-{}", Uuid::now_v7()))
      .backend(delivery::storage(database))
      .data(self.ctx.clone())
      .concurrency((self.config.subscriber_task_concurrency + self.config.system_task_concurrency).max(1) as usize)
      .enable_tracing()
      .retry(BusinessRetryPolicy { shutdown: receiver.clone() })
      .build(Self::execute);
    let worker_shutdown = receiver.clone();
    let worker_run = worker.run_until(async move {
      let mut receiver = worker_shutdown;
      receiver.wait_for(|stopped| *stopped).await.map_err(|e| std::io::Error::other(e.to_string()))?;
      Ok::<_, std::io::Error>(())
    });
    let dispatcher = async {
      let mut ticker = tokio::time::interval(Duration::from_secs(1));
      let mut receiver = receiver.clone();
      loop {
        tokio::select! {
          _ = receiver.changed() => break,
          _ = ticker.tick() => {
            super::operation::settle_cancelled(database, None).await?;
            for _ in 0..100 {
              if !delivery::dispatch_one(database).await? { break; }
            }
          }
        }
      }
      Ok::<_, RecorderError>(())
    };
    let scheduler = async {
      let mut ticker = tokio::time::interval(self.config.cron_interval_duration);
      let mut receiver = receiver.clone();
      loop {
        tokio::select! {
          _ = receiver.changed() => break,
          _ = ticker.tick() => { cron::Model::dispatch_due(database, Utc::now()).await?; }
        }
      }
      Ok::<_, RecorderError>(())
    };
    let stop = async {
      let mut receiver = receiver.clone();
      if let Some(signal) = signal {
        tokio::select! {
          _ = signal() => { shutdown.send_replace(true); }
          _ = receiver.wait_for(|stopped| *stopped) => {}
        }
      } else {
        let _ = receiver.wait_for(|stopped| *stopped).await;
      }
    };
    // Error paths also signal and join all owned loops before returning.
    let worker_run = async {
      let result = worker_run.await.map_err(|e| RecorderError::from(DbErr::Custom(e.to_string())));
      shutdown.send_replace(true);
      result
    };
    let dispatcher = async {
      let result = dispatcher.await;
      shutdown.send_replace(true);
      result
    };
    let scheduler = async {
      let result = scheduler.await;
      shutdown.send_replace(true);
      result
    };
    let (worker, dispatcher, scheduler, ()) = tokio::join!(worker_run, dispatcher, scheduler, stop);
    worker?;
    dispatcher?;
    scheduler?;
    Ok(())
  }

  pub async fn execute(envelope: TaskEnvelope, data: Data<Arc<dyn AppContextTrait>>) -> Result<(), ExecutionError> {
    let ctx = (*data).clone();
    let db = ctx.task().queue_database().map_err(|_| ExecutionError {
      retry_after: None,
      message: "Queue configuration is unavailable".into(),
    })?;
    super::operation::settle_cancelled(db, Some((&envelope.task_id, envelope.generation))).await?;
    let token = Uuid::now_v7();
    let claimed = db
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE task_runs SET status='Running', execution_token=$3, execution_lease_until=clock_timestamp()+INTERVAL '120 seconds', attempts=attempts+CASE \
         WHEN recovering THEN 0 ELSE 1 END, recovering=false WHERE id=$1 AND generation=$2 AND archived_at IS NULL AND cancel_requested_at IS NULL AND \
         ((status IN ('Pending','Scheduled') AND run_at <= clock_timestamp()) OR (status='Running' AND execution_lease_until <= clock_timestamp())) AND \
         (attempts < max_attempts OR recovering) RETURNING kind,subscriber_id,payload,payload_version,attempts,max_attempts",
        [envelope.task_id.clone().into(), envelope.generation.into(), token.into()],
      ))
      .await?;
    let Some(row) = claimed else {
      db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE task_runs SET status='Failed', done_at=clock_timestamp(), execution_token=NULL, execution_lease_until=NULL, last_error='Task execution budget \
         exhausted' WHERE id=$1 AND generation=$2 AND cancel_requested_at IS NULL AND archived_at IS NULL AND attempts >= max_attempts AND NOT recovering AND \
         (status IN ('Pending','Scheduled') OR (status='Running' AND execution_lease_until <= clock_timestamp()))",
        [envelope.task_id.clone().into(), envelope.generation.into()],
      ))
      .await?;
      let active = db
        .query_one_raw(Statement::from_sql_and_values(
          DbBackend::Postgres,
          "SELECT id FROM task_runs WHERE id=$1 AND generation=$2 AND archived_at IS NULL AND cancel_requested_at IS NULL AND status IN \
           ('Pending','Scheduled','Running')",
          [envelope.task_id.into(), envelope.generation.into()],
        ))
        .await?
        .is_some();
      if active {
        return Err(ExecutionError {
          retry_after: Some(Duration::from_secs(5)),
          message: "Task is scheduled or has a live executor".into(),
        });
      }
      return Ok(());
    };
    let kind: String = row.try_get("", "kind")?;
    let payload: serde_json::Value = row.try_get("", "payload")?;
    let owner: Option<i32> = row.try_get("", "subscriber_id")?;
    let attempts: i32 = row.try_get("", "attempts")?;
    let max_attempts: i32 = row.try_get("", "max_attempts")?;
    let fence = TaskFence {
      envelope: envelope.clone(),
      token,
    };
    // Persist a fixed migration diagnostic without exposing payload values or
    // arbitrary codec/database errors to task readers.
    let failure_message = if super::operation::legacy_jxl_options(&payload) {
      super::operation::LEGACY_JXL_DIAGNOSTIC
    } else {
      "Task execution failed"
    };
    let execution = async {
      super::operation::validate_payload(&kind, &payload, owner)?;
      if row.try_get::<i32>("", "payload_version")? != 1 {
        return Err(DbErr::Custom("Unsupported task payload version".into()).into());
      }
      if kind == "subscriber_task" {
        serde_json::from_value::<SubscriberTask>(payload)?.run_async(ctx.clone()).await
      } else {
        serde_json::from_value::<SystemTask>(payload)?.run_async(ctx.clone()).await
      }
    };
    // Both futures stay polled while renewal waits for a business fence lock.
    // This scope drops either losing future (and its transaction) before the
    // terminal UPDATE can try to acquire that same task row.
    let (result, stopping) = {
      let execution = CURRENT_FENCE.scope(fence, execution);
      let renewal = async {
        let mut ticker = tokio::time::interval_at(tokio::time::Instant::now() + Duration::from_secs(30), Duration::from_secs(30));
        loop {
          ticker.tick().await;
          let renewed = db
            .execute_raw(Statement::from_sql_and_values(
              DbBackend::Postgres,
              "UPDATE task_runs SET execution_lease_until=clock_timestamp()+INTERVAL '120 seconds' WHERE id=$1 AND generation=$2 AND execution_token=$3 AND \
               execution_lease_until > clock_timestamp() AND status='Running' AND cancel_requested_at IS NULL AND archived_at IS NULL",
              [envelope.task_id.clone().into(), envelope.generation.into(), token.into()],
            ))
            .await?;
          if renewed.rows_affected() == 0 {
            return Err::<(), RecorderError>(DbErr::Custom("Task was cancelled or lost its execution lease".into()).into());
          }
        }
      };
      let mut stopped = ctx.task().shutdown.subscribe();
      tokio::pin!(execution, renewal);
      let (mut result, stopping, aborting) = tokio::select! {
        result = &mut execution => (result, false, false),
        result = &mut renewal => (result, false, true),
        _ = stopped.wait_for(|stopped| *stopped) => (Err(DbErr::Custom("Task service stopped".into()).into()), true, true),
      };
      if aborting {
        let (business, queue) = tokio::join!(
          super::execution::cancel_queries(ctx.db().as_ref(), token),
          super::execution::cancel_queries(db, token)
        );
        if let Err(error) = business.and(queue) {
          result = Err(error.into());
        }
      }
      (result, stopping)
    };
    let retry_delay = result
      .as_ref()
      .err()
      .filter(|e| stopping || super::execution::recoverable(e))
      .filter(|_| attempts < max_attempts)
      .map(|_| if attempts == 1 { 5_i64 } else { 30_i64 });
    let status = if result.is_ok() {
      "Done"
    } else if retry_delay.is_some() {
      "Scheduled"
    } else {
      "Failed"
    };
    let completed = db
      .execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE task_runs SET status=CASE WHEN cancel_requested_at IS NOT NULL THEN 'Killed' ELSE $4 END, run_at=CASE WHEN $5::bigint IS NULL THEN run_at \
         ELSE clock_timestamp()+$5*INTERVAL '1 second' END, done_at=CASE WHEN $4='Scheduled' AND cancel_requested_at IS NULL THEN NULL ELSE clock_timestamp() \
         END, last_error=$6, execution_token=NULL, execution_lease_until=NULL WHERE id=$1 AND generation=$2 AND execution_token=$3 AND execution_lease_until \
         > clock_timestamp() AND status='Running'",
        [
          envelope.task_id.into(),
          envelope.generation.into(),
          token.into(),
          status.into(),
          retry_delay.into(),
          result.as_ref().err().map(|_| failure_message.to_owned()).into(),
        ],
      ))
      .await?;
    if completed.rows_affected() == 0 {
      return Err(ExecutionError {
        retry_after: None,
        message: "Task execution fence expired".into(),
      });
    }
    result.map_err(|_| ExecutionError {
      retry_after: retry_delay.map(|seconds| Duration::from_secs(seconds as u64)),
      message: "Task execution failed".into(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[tokio::test]
  async fn queue_construction_errors_redact_credentials() {
    let ctx = Arc::new(crate::test_utils::app::TestingAppContext::builder().build());
    let error = TaskService::from_config_and_ctx(
      TaskConfig {
        queue_database_uri: Some("postgres://SENTINEL-queue@host:invalid/queue".into()),
        ..Default::default()
      },
      ctx,
    )
    .await
    .err()
    .unwrap();
    assert!(!format!("{error:?} {error}").contains("SENTINEL"));
    assert!(error.to_string().contains("redacted"));
  }
}
