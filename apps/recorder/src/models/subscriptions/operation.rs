use sea_orm::{ColumnTrait, Condition, ConnectionTrait, DatabaseTransaction, DbBackend, DbErr, EntityTrait, QueryFilter, QueryOrder, Statement};

use super::{Column, Entity};

/// Task fences precede parent row locks. The intent lock excludes new task
/// references while existing fenced stages finish, without blocking them.
pub async fn delete_owned(db: &DatabaseTransaction, filter: Condition) -> Result<u64, DbErr> {
  let subscriptions = Entity::find().filter(filter.clone()).order_by_asc(Column::Id).all(db).await?;
  let ids: Vec<i32> = subscriptions.iter().map(|subscription| subscription.id).collect();
  for id in &ids {
    db.query_one_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "SELECT pg_advisory_xact_lock(1764, $1)",
      [(*id).into()],
    ))
    .await?;
  }
  // Acquire every affected task lock in the same order before the FK action or
  // parent DELETE; a worker can finish its current stage while we wait here.
  for id in &ids {
    let tasks = db
      .query_all_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT id FROM task_runs WHERE subscription_id=$1 ORDER BY id FOR UPDATE",
        [(*id).into()],
      ))
      .await?;
    for task in tasks {
      let task_id: String = task.try_get("", "id")?;
      db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE task_runs SET cancel_requested_at=COALESCE(cancel_requested_at,clock_timestamp()), status=CASE WHEN status IN ('Pending','Scheduled') OR \
         (status='Running' AND (execution_lease_until IS NULL OR execution_lease_until<=clock_timestamp())) THEN 'Killed' ELSE status END,done_at=CASE WHEN \
         status IN ('Pending','Scheduled') OR (status='Running' AND (execution_lease_until IS NULL OR execution_lease_until<=clock_timestamp())) THEN \
         clock_timestamp() ELSE done_at END,execution_token=CASE WHEN status='Running' AND execution_lease_until>clock_timestamp() THEN execution_token ELSE \
         NULL END,execution_lease_until=CASE WHEN status='Running' AND execution_lease_until>clock_timestamp() THEN execution_lease_until ELSE NULL END WHERE \
         id=$1 AND status IN ('Pending','Scheduled','Running')",
        [task_id.into()],
      ))
      .await?;
    }
  }
  if ids.is_empty() {
    return Ok(0);
  }
  Ok(Entity::delete_many().filter(filter).filter(Column::Id.is_in(ids)).exec(db).await?.rows_affected)
}
