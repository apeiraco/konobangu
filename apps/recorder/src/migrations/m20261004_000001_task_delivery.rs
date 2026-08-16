use sea_orm::DbBackend;
use sea_orm_migration::prelude::*;

use super::defs::{ApalisJobs, Cron, Subscribers, Subscriptions, TaskOutbox, TaskRuns, TaskViews};
use crate::database::roles::TASK_CONTROL_ACCESS_ROLE;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared(&schema_sql()).await?;
    // Version 0 used the same registered task variants. Decode and serialize
    // through those types explicitly rather than treating arbitrary JSON as v1.
    let db = manager.get_connection();
    for row in db
      .query_all_raw(
        DbBackend::Postgres.build(
          &Query::select()
            .columns([TaskRuns::Id, TaskRuns::Kind, TaskRuns::Payload])
            .from(TaskRuns::Table)
            .to_owned(),
        ),
      )
      .await?
    {
      let id: String = row.try_get("", &TaskRuns::Id.to_string())?;
      let kind: String = row.try_get("", &TaskRuns::Kind.to_string())?;
      let payload: serde_json::Value = row.try_get("", &TaskRuns::Payload.to_string())?;
      let converted = match kind.as_str() {
        "subscriber_task" => serde_json::from_value::<crate::task::SubscriberTask>(payload).and_then(serde_json::to_value),
        "system_task" => serde_json::from_value::<crate::task::SystemTask>(payload).and_then(serde_json::to_value),
        _ => return Err(DbErr::Migration(format!("Task {id}: unsupported legacy payload kind"))),
      }
      .map_err(|_| DbErr::Migration(format!("Task {id}: legacy payload conversion failed")))?;
      db.execute_raw(
        DbBackend::Postgres.build(
          &Query::update()
            .table(TaskRuns::Table)
            .values([(TaskRuns::Payload, converted.into()), (TaskRuns::PayloadVersion, 1.into())])
            .and_where(Expr::col(TaskRuns::Id).eq(id))
            .to_owned(),
        ),
      )
      .await?;
    }
    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    Err(DbErr::Migration(
      "Task cutover is not reversible after new writes; stop services and restore the pre-upgrade backup".into(),
    ))
  }
}

// PostgreSQL policies, partial indexes and the legacy cutover need native SQL.
// Names come from the same Rust identifiers as the typed migration queries.
fn schema_sql() -> String {
  format!(
    r#"
CREATE TABLE {task_runs} (
  {id} text PRIMARY KEY,
  {kind} text NOT NULL CHECK ({kind} IN ('subscriber_task', 'system_task')),
  {subscriber_id} integer REFERENCES {subscribers}({subscribers_id}) ON DELETE RESTRICT,
  {subscription_id} integer REFERENCES {subscriptions}({subscriptions_id}) ON DELETE SET NULL,
  {cron_id} integer REFERENCES {cron}({cron_target_id}) ON DELETE SET NULL,
  {scheduled_at_utc} timestamptz,
  {payload_version} integer NOT NULL DEFAULT 1 CHECK ({payload_version} = 1),
  {payload} jsonb NOT NULL,
  {task_type} text NOT NULL,
  {status} text NOT NULL DEFAULT 'Pending' CHECK ({status} IN ('Pending','Scheduled','Running','Done','Failed','Killed')),
  {generation} bigint NOT NULL DEFAULT 1 CHECK ({generation} > 0),
  {attempts} integer NOT NULL DEFAULT 0 CHECK ({attempts} >= 0),
  {max_attempts} integer NOT NULL DEFAULT 3 CHECK ({max_attempts} > 0),
  {run_at} timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  {done_at} timestamptz,
  {last_error} text,
  {cancel_requested_at} timestamptz,
  {archived_at} timestamptz,
  {execution_token} uuid,
  {execution_lease_until} timestamptz,
  {recovering} boolean NOT NULL DEFAULT false,
  {legacy_execution} jsonb,
  {created_at} timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK ({kind} <> 'subscriber_task' OR {subscriber_id} IS NOT NULL),
  UNIQUE ({cron_id}, {scheduled_at_utc})
);
CREATE INDEX {task_runs}_owner ON {task_runs}({subscriber_id}, {run_at} DESC) WHERE {archived_at} IS NULL;
CREATE INDEX {task_runs}_recovery ON {task_runs}({execution_lease_until}) WHERE {status} = 'Running';
CREATE TABLE {task_outbox} (
  {task_id} text NOT NULL REFERENCES {task_runs}({id}) ON DELETE RESTRICT,
  {generation} bigint NOT NULL,
  {available_at} timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  {lease_token} uuid,
  {lease_until} timestamptz,
  {attempts} integer NOT NULL DEFAULT 0,
  {last_error} text,
  {sent_at} timestamptz,
  PRIMARY KEY ({task_id}, {generation})
);
CREATE INDEX {task_outbox}_due ON {task_outbox}({available_at}) WHERE {sent_at} IS NULL;

INSERT INTO {task_runs} ({id}, {kind}, {subscriber_id}, {subscription_id}, {cron_id}, {scheduled_at_utc}, {payload}, {task_type},
  {status}, {attempts}, {max_attempts}, {run_at}, {done_at}, {last_error}, {recovering}, {legacy_execution})
SELECT {id}, {job_type}, {subscriber_id}, {subscription_id}, {cron_id}, CASE WHEN {cron_id} IS NOT NULL THEN {run_at} ELSE NULL END, {job}, {task_type},
  CASE WHEN {status} = 'Running' THEN 'Pending' ELSE {status} END,
  {attempts}, {max_attempts}, {run_at}, {done_at}, {last_error}, {status} = 'Running',
  jsonb_build_object('status', {status}, 'attempts', {attempts}, 'lock_at', {lock_at}, 'lock_by', {lock_by}, 'priority', {priority})
FROM apalis.jobs;
INSERT INTO {task_outbox} ({task_id}, {generation}, {available_at})
SELECT {id}, {generation}, {run_at} FROM {task_runs} WHERE {status} IN ('Pending', 'Scheduled');

UPDATE {cron} SET {status}='pending', {locked_at}=NULL, {locked_by}=NULL, {attempts}=0 WHERE {status}='running';
DROP VIEW {subscriber_tasks};
DROP VIEW {system_tasks};
DROP TRIGGER setup_apalis_jobs_extra_foreign_keys_trigger ON apalis.jobs;
DROP FUNCTION apalis.setup_apalis_jobs_extra_foreign_keys();
DROP TRIGGER notify_due_cron_when_mutating_trigger ON {cron};
DROP FUNCTION notify_due_cron_when_mutating();
DROP FUNCTION check_and_trigger_due_crons();
ALTER SCHEMA apalis RENAME TO apalis_legacy_v07;
DO $$ BEGIN
  IF to_regclass('public._sqlx_migrations') IS NOT NULL THEN
    ALTER TABLE public._sqlx_migrations SET SCHEMA apalis_legacy_v07;
  END IF;
END $$;
REVOKE ALL ON SCHEMA apalis_legacy_v07 FROM PUBLIC;
DO $$ DECLARE role_name text; BEGIN
  FOR role_name IN SELECT rolname FROM pg_roles WHERE NOT rolsuper AND rolname <> current_user LOOP
    EXECUTE format('REVOKE ALL ON SCHEMA apalis_legacy_v07 FROM %I', role_name);
    EXECUTE format('REVOKE ALL ON ALL TABLES IN SCHEMA apalis_legacy_v07 FROM %I', role_name);
    EXECUTE format('REVOKE ALL ON ALL SEQUENCES IN SCHEMA apalis_legacy_v07 FROM %I', role_name);
    EXECUTE format('REVOKE ALL ON ALL FUNCTIONS IN SCHEMA apalis_legacy_v07 FROM %I', role_name);
  END LOOP;
END $$;

ALTER TABLE {task_runs} ENABLE ROW LEVEL SECURITY;
ALTER TABLE {task_runs} FORCE ROW LEVEL SECURITY;
CREATE POLICY {task_runs}_app_scoped ON {task_runs} USING (
  {subscriber_id} = NULLIF(current_setting('app.subscriber_id', true), '')::integer
) WITH CHECK ({subscriber_id} = NULLIF(current_setting('app.subscriber_id', true), '')::integer);
CREATE POLICY {task_runs}_task_control ON {task_runs} TO {task_control_access} USING (true) WITH CHECK (true);
ALTER TABLE {task_outbox} ENABLE ROW LEVEL SECURITY;
ALTER TABLE {task_outbox} FORCE ROW LEVEL SECURITY;
CREATE POLICY {task_outbox}_app_scoped ON {task_outbox} USING (
  EXISTS (SELECT 1 FROM {task_runs} t WHERE t.{id} = {task_id} AND t.{subscriber_id} = NULLIF(current_setting('app.subscriber_id', true), '')::integer)
) WITH CHECK (
  EXISTS (SELECT 1 FROM {task_runs} t WHERE t.{id} = {task_id} AND t.{subscriber_id} = NULLIF(current_setting('app.subscriber_id', true), '')::integer)
);
CREATE POLICY {task_outbox}_task_control ON {task_outbox} TO {task_control_access} USING (true) WITH CHECK (true);
CREATE VIEW {subscriber_tasks} WITH (security_invoker = true) AS
  SELECT {id}, {subscriber_id}, {subscription_id}, {cron_id}, {payload} AS {job}, {task_type},
    {status}, {generation}, {attempts}, {max_attempts}, {run_at}, {last_error}, {done_at}, {cancel_requested_at}
  FROM {task_runs} WHERE {kind} = 'subscriber_task' AND {archived_at} IS NULL;
CREATE VIEW {system_tasks} WITH (security_invoker = true) AS
  SELECT {id}, {subscriber_id}, {cron_id}, {payload} AS {job}, {task_type},
    {status}, {generation}, {attempts}, {max_attempts}, {run_at}, {last_error}, {done_at}, {cancel_requested_at}
  FROM {task_runs} WHERE {kind} = 'system_task' AND {archived_at} IS NULL;
"#,
    archived_at = TaskRuns::ArchivedAt.to_string(),
    attempts = TaskRuns::Attempts.to_string(),
    available_at = TaskOutbox::AvailableAt.to_string(),
    cancel_requested_at = TaskRuns::CancelRequestedAt.to_string(),
    created_at = TaskRuns::CreatedAt.to_string(),
    cron = Cron::Table.to_string(),
    cron_id = TaskRuns::CronId.to_string(),
    cron_target_id = Cron::Id.to_string(),
    done_at = TaskRuns::DoneAt.to_string(),
    execution_lease_until = TaskRuns::ExecutionLeaseUntil.to_string(),
    execution_token = TaskRuns::ExecutionToken.to_string(),
    generation = TaskRuns::Generation.to_string(),
    id = TaskRuns::Id.to_string(),
    job = ApalisJobs::Job.to_string(),
    job_type = ApalisJobs::JobType.to_string(),
    kind = TaskRuns::Kind.to_string(),
    last_error = TaskRuns::LastError.to_string(),
    lease_token = TaskOutbox::LeaseToken.to_string(),
    lease_until = TaskOutbox::LeaseUntil.to_string(),
    legacy_execution = TaskRuns::LegacyExecution.to_string(),
    lock_at = ApalisJobs::LockAt.to_string(),
    lock_by = ApalisJobs::LockBy.to_string(),
    locked_at = Cron::LockedAt.to_string(),
    locked_by = Cron::LockedBy.to_string(),
    max_attempts = TaskRuns::MaxAttempts.to_string(),
    payload = TaskRuns::Payload.to_string(),
    payload_version = TaskRuns::PayloadVersion.to_string(),
    priority = ApalisJobs::Priority.to_string(),
    recovering = TaskRuns::Recovering.to_string(),
    run_at = TaskRuns::RunAt.to_string(),
    scheduled_at_utc = TaskRuns::ScheduledAtUtc.to_string(),
    sent_at = TaskOutbox::SentAt.to_string(),
    status = TaskRuns::Status.to_string(),
    subscriber_id = TaskRuns::SubscriberId.to_string(),
    subscriber_tasks = TaskViews::SubscriberTasks.to_string(),
    subscribers = Subscribers::Table.to_string(),
    subscribers_id = Subscribers::Id.to_string(),
    subscription_id = TaskRuns::SubscriptionId.to_string(),
    subscriptions = Subscriptions::Table.to_string(),
    subscriptions_id = Subscriptions::Id.to_string(),
    system_tasks = TaskViews::SystemTasks.to_string(),
    task_id = TaskOutbox::TaskId.to_string(),
    task_outbox = TaskOutbox::Table.to_string(),
    task_runs = TaskRuns::Table.to_string(),
    task_type = TaskRuns::TaskType.to_string(),
    task_control_access = TASK_CONTROL_ACCESS_ROLE,
  )
}
