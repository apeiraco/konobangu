use apalis::prelude::*;
use apalis_postgres::{Config, PostgresStorage};
use chrono::Utc;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement, TransactionTrait};
use uuid::Uuid;

use super::execution::TaskEnvelope;

pub const DELIVERY_QUEUE: &str = "konobangu.business.v1";
pub fn storage(db: &DatabaseConnection) -> PostgresStorage<TaskEnvelope> {
  PostgresStorage::new(db.get_postgres_connection_pool()).with_config(
    Config::default()
      .queue(DELIVERY_QUEUE)
      .heartbeat_interval(std::time::Duration::from_secs(30))
      .missed_heartbeats(2),
  )
}

pub async fn dispatch_one(db: &DatabaseConnection) -> Result<bool, DbErr> {
  let token = Uuid::now_v7();
  let transaction = db.begin().await?;
  let claimed = transaction
    .query_one_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "WITH due AS (SELECT task_id,generation FROM task_outbox WHERE sent_at IS NULL AND available_at <= clock_timestamp() AND (lease_until IS NULL OR \
       lease_until <= clock_timestamp()) ORDER BY available_at FOR UPDATE SKIP LOCKED LIMIT 1) UPDATE task_outbox o SET lease_token=$1, \
       lease_until=clock_timestamp()+INTERVAL '60 seconds', attempts=o.attempts+1 FROM due WHERE o.task_id=due.task_id AND o.generation=due.generation \
       RETURNING o.task_id,o.generation,o.attempts",
      [token.into()],
    ))
    .await?;
  transaction.commit().await?;
  let Some(claimed) = claimed else {
    return Ok(false);
  };
  let id: String = claimed.try_get("", "task_id")?;
  let generation: i64 = claimed.try_get("", "generation")?;
  let attempts: i32 = claimed.try_get("", "attempts")?;
  let effective = db
    .query_one_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "SELECT id FROM task_runs WHERE id=$1 AND generation=$2 AND status IN ('Pending','Scheduled','Running') AND cancel_requested_at IS NULL AND archived_at \
       IS NULL",
      [id.clone().into(), generation.into()],
    ))
    .await?
    .is_some();
  // No application transaction remains open during queue I/O.
  let delivered = if effective {
    let envelope = TaskEnvelope {
      task_id: id.clone(),
      generation,
    };
    let task = TaskBuilder::new(envelope)
      .idempotency_key(format!("{id}:{generation}"))
      .max_attempts(1000)
      .run_at_time(Utc::now().into())
      .build();
    match storage(db).push_task(task).await {
      Ok(()) => Ok(()),
      Err(apalis::prelude::TaskSinkError::PushError(apalis_postgres::Error::Database(sqlx::Error::Database(error))))
        if error.code().as_deref() == Some("23505") && error.constraint() == Some("idx_jobs_idempotency_key") =>
      {
        Ok(())
      }
      Err(error) => Err(error.to_string()),
    }
  } else {
    Ok(())
  };
  let transaction = db.begin().await?;
  match delivered {
    Ok(()) => {
      transaction
        .execute_raw(Statement::from_sql_and_values(
          DbBackend::Postgres,
          "UPDATE task_outbox SET sent_at=clock_timestamp(), lease_until=NULL, lease_token=NULL, last_error=NULL WHERE task_id=$1 AND generation=$2 AND \
           lease_token=$3",
          [id.into(), generation.into(), token.into()],
        ))
        .await?;
    }
    Err(_) => {
      // Queue errors may contain connection details; persist only a safe
      // category.
      let delay = 2_i64.pow((attempts.saturating_sub(1) as u32).min(6)).min(60);
      transaction
        .execute_raw(Statement::from_sql_and_values(
          DbBackend::Postgres,
          "UPDATE task_outbox SET available_at=clock_timestamp()+$4*INTERVAL '1 second', lease_until=NULL, lease_token=NULL, last_error='Queue delivery \
           failed' WHERE task_id=$1 AND generation=$2 AND lease_token=$3",
          [id.into(), generation.into(), token.into(), delay.into()],
        ))
        .await?;
    }
  }
  transaction.commit().await?;
  Ok(true)
}
