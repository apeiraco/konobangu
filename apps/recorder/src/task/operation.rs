use sea_orm::{ConnectionTrait, DatabaseTransaction, DbBackend, DbErr, Statement};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use super::{SubscriberTask, SubscriberTaskTrait, SystemTask, SystemTaskTrait};

fn invalid(message: &str) -> DbErr {
  DbErr::Custom(message.into())
}

pub(super) const LEGACY_JXL_DIAGNOSTIC: &str =
  "format_options: legacy libjxl distance/effort/quality/speed/progressive require operator migration; use preset_version=1";

pub(super) fn legacy_jxl_options(payload: &Value) -> bool {
  payload.get("task_type").and_then(Value::as_str) == Some("optimize_image")
    && payload.get("format_options").and_then(|o| o.get("mime_type")).and_then(Value::as_str) == Some("image/jxl")
    && ["distance", "effort", "quality", "speed", "progressive"]
      .iter()
      .any(|field| payload["format_options"].get(field).is_some())
}

pub fn validate_payload(kind: &str, payload: &Value, owner: Option<i32>) -> Result<(), DbErr> {
  if legacy_jxl_options(payload) {
    return Err(invalid(LEGACY_JXL_DIAGNOSTIC));
  }
  match kind {
    "subscriber_task" => {
      let task: SubscriberTask = serde_json::from_value(payload.clone()).map_err(|_| invalid("Invalid subscriber task input"))?;
      if Some(task.get_subscriber_id()) != owner || owner.is_none_or(|id| id <= 0) {
        return Err(invalid("Invalid task owner"));
      }
    }
    "system_task" => {
      let task: SystemTask = serde_json::from_value(payload.clone()).map_err(|_| invalid("Invalid system task input"))?;
      if task.get_subscriber_id() != owner || owner.is_some_and(|id| id <= 0) {
        return Err(invalid("Invalid task owner"));
      }
    }
    _ => return Err(invalid("Unknown task kind")),
  }
  Ok(())
}

async fn check_reference(db: &DatabaseTransaction, payload: &Value, kind: &str) -> Result<(), DbErr> {
  if let Some(subscription) = payload.get("subscription_id").and_then(Value::as_i64) {
    lock_subscription_intent(db, subscription).await?;
    let visible = db
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT id FROM subscriptions WHERE id = $1 AND subscriber_id = $2",
        [subscription.into(), payload.get("subscriber_id").and_then(Value::as_i64).into()],
      ))
      .await?;
    if visible.is_none() {
      return Err(DbErr::RecordNotFound("Task subscription is unavailable to this owner".into()));
    }
  }
  if let Some(cron) = payload.get("cron_id").and_then(Value::as_i64) {
    let visible = db
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT id FROM cron WHERE id = $1 AND subscriber_id IS NOT DISTINCT FROM $2 AND (CASE WHEN subscriber_task_cron IS NOT NULL THEN 'subscriber_task' \
         ELSE 'system_task' END) = $3",
        [cron.into(), payload.get("subscriber_id").and_then(Value::as_i64).into(), kind.into()],
      ))
      .await?;
    if visible.is_none() {
      return Err(DbErr::RecordNotFound("Task cron is unavailable to this owner".into()));
    }
  }
  Ok(())
}

/// The caller owns the transaction, including later GraphQL roots and batches.
pub async fn enqueue<T: Serialize>(db: &DatabaseTransaction, kind: &str, task: &T) -> Result<String, DbErr> {
  let payload = serde_json::to_value(task).map_err(|_| invalid("Invalid task serialization"))?;
  check_reference(db, &payload, kind).await?;
  enqueue_payload(db, kind, payload, None).await
}

/// Only the restricted scheduler calls this with a locked, trusted cron record.
pub async fn enqueue_cron<T: Serialize>(db: &DatabaseTransaction, kind: &str, task: &T, scheduled: chrono::DateTime<chrono::Utc>) -> Result<String, DbErr> {
  let payload = serde_json::to_value(task).map_err(|_| invalid("Invalid cron task serialization"))?;
  enqueue_payload(db, kind, payload, Some(scheduled)).await
}

async fn enqueue_payload(db: &DatabaseTransaction, kind: &str, payload: Value, scheduled: Option<chrono::DateTime<chrono::Utc>>) -> Result<String, DbErr> {
  let owner = payload
    .get("subscriber_id")
    .and_then(Value::as_i64)
    .map(|id| i32::try_from(id).map_err(|_| invalid("Invalid task owner")))
    .transpose()?;
  validate_payload(kind, &payload, owner)?;
  if let Some(subscription) = payload.get("subscription_id").and_then(Value::as_i64) {
    lock_subscription_intent(db, subscription).await?;
  }
  let id = Uuid::now_v7().to_string();
  let task_type = payload
    .get("task_type")
    .and_then(Value::as_str)
    .ok_or_else(|| invalid("Missing task type"))?
    .to_owned();
  let subscription = payload.get("subscription_id").and_then(Value::as_i64);
  let cron = payload.get("cron_id").and_then(Value::as_i64);
  db.execute_raw(Statement::from_sql_and_values(
    DbBackend::Postgres,
    "INSERT INTO task_runs (id, kind, subscriber_id, subscription_id, cron_id, scheduled_at_utc, payload, task_type) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    [
      id.clone().into(),
      kind.to_owned().into(),
      owner.into(),
      subscription.into(),
      cron.into(),
      scheduled.into(),
      payload.into(),
      task_type.into(),
    ],
  ))
  .await?;
  db.execute_raw(Statement::from_sql_and_values(
    DbBackend::Postgres,
    "INSERT INTO task_outbox (task_id, generation) VALUES ($1,1)",
    [id.clone().into()],
  ))
  .await?;
  Ok(id)
}

pub async fn retry(db: &DatabaseTransaction, id: &str) -> Result<(), DbErr> {
  let task = db
    .query_one_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "SELECT kind,payload,subscriber_id,status FROM task_runs WHERE id = $1 AND archived_at IS NULL FOR UPDATE",
      [id.to_owned().into()],
    ))
    .await?
    .ok_or_else(|| invalid("Task is unavailable"))?;
  let status: String = task.try_get("", "status")?;
  if !matches!(status.as_str(), "Failed" | "Killed") {
    return Err(invalid("Task retry conflicts with its current state"));
  }
  let payload: Value = task.try_get("", "payload")?;
  let kind: String = task.try_get("", "kind")?;
  validate_payload(&kind, &payload, task.try_get("", "subscriber_id")?)?;
  check_reference(db, &payload, &kind).await?;
  let updated = db
    .query_one_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "UPDATE task_runs SET generation = generation+1, status = 'Pending', attempts = 0, run_at = CURRENT_TIMESTAMP, done_at = NULL, last_error = NULL, \
       cancel_requested_at = NULL, execution_token = NULL, execution_lease_until = NULL, recovering = false WHERE id = $1 AND (execution_lease_until IS NULL \
       OR execution_lease_until <= clock_timestamp()) RETURNING generation",
      [id.to_owned().into()],
    ))
    .await?
    .ok_or_else(|| invalid("Task still has an active executor"))?;
  db.execute_raw(Statement::from_sql_and_values(
    DbBackend::Postgres,
    "INSERT INTO task_outbox (task_id,generation) VALUES ($1,$2)",
    [id.to_owned().into(), updated.try_get::<i64>("", "generation")?.into()],
  ))
  .await?;
  Ok(())
}

/// A nonblocking shared intent prevents inserts/retries racing subscription
/// deletion without inverting an already held task/cron fence lock.
pub async fn lock_subscription_intent(db: &DatabaseTransaction, id: i64) -> Result<(), DbErr> {
  if !try_lock_subscription_intent(db, id).await? {
    return Err(invalid("Subscription deletion conflicts with task creation or retry"));
  }
  Ok(())
}

pub async fn try_lock_subscription_intent(db: &DatabaseTransaction, id: i64) -> Result<bool, DbErr> {
  let id = i32::try_from(id).map_err(|_| invalid("Invalid subscription ID"))?;
  let locked = db
    .query_one_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "SELECT pg_try_advisory_xact_lock_shared(1764, $1) AS locked",
      [id.into()],
    ))
    .await?
    .unwrap();
  locked.try_get("", "locked")
}

/// Cancellation wins over budget exhaustion, without requiring another queue
/// delivery. Skip held fences and revisit them on the existing dispatcher tick.
pub async fn settle_cancelled<C: ConnectionTrait>(db: &C, envelope: Option<(&str, i64)>) -> Result<u64, DbErr> {
  let (id, generation) = envelope.map(|(id, generation)| (Some(id.to_owned()), Some(generation))).unwrap_or((None, None));
  Ok(
    db.execute_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "WITH expired AS (SELECT id,generation,execution_token FROM task_runs WHERE status='Running' AND cancel_requested_at IS NOT NULL AND \
       (execution_lease_until IS NULL OR execution_lease_until <= clock_timestamp()) AND ($1::text IS NULL OR (id=$1 AND generation=$2)) ORDER BY id FOR \
       UPDATE SKIP LOCKED) UPDATE task_runs t SET status='Killed',done_at=clock_timestamp(),execution_token=NULL,execution_lease_until=NULL FROM expired e \
       WHERE t.id=e.id AND t.generation=e.generation AND t.execution_token IS NOT DISTINCT FROM e.execution_token AND t.status='Running' AND \
       t.cancel_requested_at IS NOT NULL AND (t.execution_lease_until IS NULL OR t.execution_lease_until <= clock_timestamp())",
      [id.into(), generation.into()],
    ))
    .await?
    .rows_affected(),
  )
}

pub async fn cancel_or_archive(db: &DatabaseTransaction, id: &str) -> Result<u64, DbErr> {
  let updated = db
    .execute_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "UPDATE task_runs SET cancel_requested_at = COALESCE(cancel_requested_at, clock_timestamp()), status = CASE WHEN status IN ('Pending','Scheduled') THEN \
       'Killed' ELSE status END, archived_at = CASE WHEN status IN ('Done','Failed','Killed') THEN clock_timestamp() ELSE archived_at END, done_at = CASE \
       WHEN status IN ('Pending','Scheduled') THEN clock_timestamp() ELSE done_at END WHERE id = $1 AND archived_at IS NULL",
      [id.to_owned().into()],
    ))
    .await?;
  // Reuse the same generation/token/lease rule as polling and replay.
  if updated.rows_affected() != 0 {
    let generation = db
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT generation FROM task_runs WHERE id=$1",
        [id.to_owned().into()],
      ))
      .await?
      .unwrap()
      .try_get("", "generation")?;
    settle_cancelled(db, Some((id, generation))).await?;
  }
  Ok(updated.rows_affected())
}
