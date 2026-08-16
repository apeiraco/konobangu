//! Owner-only bootstrap and preflight for the immutable 0.7 queue schema.
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement};
use sea_orm_migration::MigratorTrait;
use serde::Serialize;

use crate::task::{SubscriberTask, SubscriberTaskTrait, SystemTask, SystemTaskTrait};

#[derive(Debug, Default, Serialize)]
pub struct PreflightReport {
  pub legacy_tasks: usize,
  pub issues: Vec<String>,
}

pub async fn preflight(db: &DatabaseConnection) -> Result<PreflightReport, DbErr> {
  let mut report = PreflightReport::default();
  let exists = db
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT to_regclass('apalis.jobs') IS NOT NULL AS legacy, to_regclass('public._sqlx_migrations') IS NOT NULL AS history, \
       to_regnamespace('apalis_legacy_v07') IS NOT NULL AS archived, to_regnamespace('apalis') IS NOT NULL AS queue_schema, to_regclass('apalis.workers') IS \
       NOT NULL AS workers",
    ))
    .await?
    .ok_or_else(|| DbErr::Custom("Queue preflight failed".into()))?;
  let business_history = db
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT to_regclass('public.seaql_migrations') IS NOT NULL AS present",
    ))
    .await?
    .unwrap()
    .try_get::<bool>("", "present")?;
  if business_history {
    let known = super::Migrator::migrations();
    let applied = db
      .query_all_raw(Statement::from_string(
        DbBackend::Postgres,
        "SELECT version FROM seaql_migrations ORDER BY version",
      ))
      .await?;
    for (index, row) in applied.iter().enumerate() {
      let version: String = row.try_get("", "version")?;
      if known.get(index).is_none_or(|migration| migration.name() != version) {
        report
          .issues
          .push(format!("Business migration version {version} is unknown or history is not a contiguous prefix"));
      }
    }
  }

  if exists.try_get::<bool>("", "history")? {
    for row in db
      .query_all_raw(Statement::from_string(
        DbBackend::Postgres,
        "SELECT version, checksum, success FROM public._sqlx_migrations",
      ))
      .await?
    {
      let version: i64 = row.try_get("", "version")?;
      let checksum: Vec<u8> = row.try_get("", "checksum")?;
      if !HISTORICAL_CHECKSUMS
        .iter()
        .any(|(known, digest)| *known == version && hex_checksum(&checksum) == *digest)
        || !row.try_get::<bool>("", "success").unwrap_or(false)
      {
        report
          .issues
          .push(format!("SQLx history version {version} is not a confirmed successful legacy queue migration"));
      }
    }
  }
  if exists.try_get::<bool>("", "archived")? {
    return Ok(report);
  }
  if !exists.try_get::<bool>("", "legacy")? {
    if exists.try_get::<bool>("", "queue_schema")? {
      report
        .issues
        .push("Unrecognized apalis schema without legacy jobs: inspect before historical bootstrap".into());
    }
    return Ok(report);
  }
  if !exists.try_get::<bool>("", "workers")? {
    report.issues.push("Legacy queue schema is missing workers; liveness cannot be verified".into());
  }
  let columns = db
    .query_all_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT column_name FROM information_schema.columns WHERE table_schema='apalis' AND table_name='jobs'",
    ))
    .await?;
  let columns: std::collections::HashSet<String> = columns.iter().map(|row| row.try_get("", "column_name")).collect::<Result<_, _>>()?;
  for required in [
    "id",
    "job",
    "job_type",
    "status",
    "attempts",
    "max_attempts",
    "run_at",
    "done_at",
    "last_error",
    "lock_at",
    "lock_by",
    "priority",
  ] {
    if !columns.contains(required) {
      report.issues.push(format!("Legacy queue schema is missing column {required}"));
    }
  }
  if !report.issues.is_empty() {
    return Ok(report);
  }
  for row in db
    .query_all_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT id FROM apalis.jobs GROUP BY id HAVING count(*) > 1",
    ))
    .await?
  {
    report.issues.push(format!("Task {}: duplicate legacy ID", row.try_get::<String>("", "id")?));
  }
  for row in db
    .query_all_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT job->>'cron_id' AS cron_id, run_at FROM apalis.jobs WHERE job->>'cron_id' IS NOT NULL GROUP BY job->>'cron_id',run_at HAVING count(*)>1",
    ))
    .await?
  {
    report
      .issues
      .push(format!("Cron {}: duplicate legacy UTC occurrence", row.try_get::<String>("", "cron_id")?));
  }
  let live = db
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT count(*) AS count FROM apalis.workers WHERE last_seen > CURRENT_TIMESTAMP - INTERVAL '2 minutes'",
    ))
    .await?
    .ok_or_else(|| DbErr::Custom("Worker preflight failed".into()))?;
  if live.try_get::<i64>("", "count")? > 0 {
    report
      .issues
      .push("Recent legacy worker heartbeat: stop all writers/workers and wait for liveness to expire".into());
  }
  for row in db
    .query_all_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT id, job, job_type, status, attempts, max_attempts, to_jsonb(j) AS original FROM apalis.jobs j",
    ))
    .await?
  {
    report.legacy_tasks += 1;
    let id: String = row.try_get("", "id")?;
    let kind: String = row.try_get("", "job_type")?;
    let payload: serde_json::Value = row.try_get("", "job")?;
    let status: String = row.try_get("", "status")?;
    if !matches!(status.as_str(), "Pending" | "Scheduled" | "Running" | "Done" | "Failed" | "Killed") {
      report.issues.push(format!("Task {id}: unknown status"));
    }
    if row.try_get::<i32>("", "attempts")? < 0 || row.try_get::<i32>("", "max_attempts")? <= 0 {
      report.issues.push(format!("Task {id}: invalid attempt budget"));
    }
    let attribution = match kind.as_str() {
      "subscriber_task" => serde_json::from_value::<SubscriberTask>(payload.clone())
        .ok()
        .map(|task| (Some(task.get_subscriber_id()), task.get_cron_id())),
      "system_task" => serde_json::from_value::<SystemTask>(payload.clone())
        .ok()
        .map(|task| (task.get_subscriber_id(), task.get_cron_id())),
      _ => None,
    };
    let Some((owner, cron)) = attribution else {
      report.issues.push(format!("Task {id}: unknown kind or invalid version-0 payload"));
      continue;
    };
    let original: serde_json::Value = row.try_get("", "original")?;
    for (field, expected) in [
      ("subscriber_id", owner.map(i64::from)),
      ("cron_id", cron.map(i64::from)),
      ("subscription_id", payload.get("subscription_id").and_then(|v| v.as_i64())),
    ] {
      if original.get(field).is_some_and(|actual| actual.as_i64() != expected) {
        report.issues.push(format!("Task {id}: denormalized {field} conflicts with its payload"));
      }
    }
    if original.get("task_type").is_some_and(|actual| Some(actual) != payload.get("task_type")) {
      report.issues.push(format!("Task {id}: denormalized task type conflicts with its payload"));
    }
    if let Some(owner) = owner {
      let present = db
        .query_one_raw(Statement::from_sql_and_values(
          DbBackend::Postgres,
          "SELECT id FROM subscribers WHERE id = $1",
          [owner.into()],
        ))
        .await?;
      if owner <= 0 || present.is_none() {
        report.issues.push(format!("Task {id}: owner cannot be attributed"));
      }
    }
    if let Some(subscription) = payload.get("subscription_id").and_then(|v| v.as_i64()) {
      let present = db
        .query_one_raw(Statement::from_sql_and_values(
          DbBackend::Postgres,
          "SELECT id FROM subscriptions WHERE id = $1 AND subscriber_id = $2",
          [subscription.into(), owner.into()],
        ))
        .await?;
      if present.is_none() {
        report.issues.push(format!("Task {id}: subscription is missing or belongs to another owner"));
      }
    }
    if let Some(cron) = cron {
      let present = db
        .query_one_raw(Statement::from_sql_and_values(
          DbBackend::Postgres,
          "SELECT id FROM cron WHERE id = $1 AND subscriber_id IS NOT DISTINCT FROM $2",
          [cron.into(), owner.into()],
        ))
        .await?;
      if present.is_none() {
        report.issues.push(format!("Task {id}: cron is missing or belongs to another owner"));
      }
    }
  }
  Ok(report)
}

pub async fn bootstrap(db: &DatabaseConnection) -> Result<(), DbErr> {
  use sea_orm::TransactionTrait;
  use sea_orm_migration::{MigrationTrait, SchemaManager};
  let transaction = db.begin().await?;
  // Serialize concurrent owner bootstrap attempts without changing old
  // histories.
  transaction.execute_unprepared("SELECT pg_advisory_xact_lock(610040001)").await?;
  let exists = transaction
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT to_regnamespace('apalis_legacy_v07') IS NOT NULL OR to_regclass('apalis.jobs') IS NOT NULL AS present",
    ))
    .await?
    .unwrap()
    .try_get::<bool>("", "present")?;
  if !exists {
    super::legacy_schema::Migration.up(&SchemaManager::new(&transaction)).await?;
  }
  transaction.commit().await
}

pub async fn initialize_queue(db: &DatabaseConnection) -> Result<(), DbErr> {
  // Explicitly apply the published sqlx.toml settings to the public migrator.
  let mut migrator = apalis_postgres::PostgresStorage::<()>::migrations();
  migrator.dangerous_set_table_name("apalis._sqlx_migrations");
  migrator.create_schema("apalis");
  migrator
    .run(db.get_postgres_connection_pool())
    .await
    .map_err(|e| DbErr::Migration(e.to_string()))?;
  Ok(())
}

// SHA-384 fingerprints of published apalis-sql 0.7.4 migrations. Existing
// SQLx history is validated, never replayed or rewritten by this application.
const HISTORICAL_CHECKSUMS: &[(i64, &str)] = &[
  (
    20220530084123,
    "0e2ae5fd117c32925d7ff4ba4f26368269fba7706bf1c3e9578a763a3e2acc5fd44c8edf2aa25e7c0ac1c6ebc8e95a9a",
  ),
  (
    20220709210445,
    "14f906e53a627f1b27996e74e8e69dbc29ee70519e7af4f0032a27c75ed706a34af8bafecc8f8572855f314cbc5a7ba9",
  ),
  (
    20230330210841,
    "d9b6e57e6de60db1647e2447cf36bf0eafbec266f1d482bd196affc97617efbc4c07bad826706c7b3a0b3960cf9f727b",
  ),
  (
    20230408110421,
    "3656a91fe9c652ca23df775a601409a9252357f47088d6bcf2025fda2dd67e50c99ad94d1d2ca569c91c343189c58e33",
  ),
  (
    20230408234928,
    "176c3acaf1097ff45da76d4c746af766640027b6721a056b3fc3fcc4ab8192344920b4415e8dae46f5136ee223ed5f6b",
  ),
  (
    20240225141841,
    "c53560b7a312915d696ed6c7e18bda478d9bf1278c4d59e3c53b47c45d8a903f089c728a5ff2e131cd5b629cdfe7d4cb",
  ),
  (
    20250210092135,
    "810786bd8f1007bf523bec8ed9de0a9ae789e634e5434a1a5cc9c1b766bd758b4d37912ef390dcf95e8de23910eeef55",
  ),
  (
    20250223193249,
    "cfda3f3d62b4122b6321af0e45ba162fd3ad5d07f95e4eefe858600551167e31454ebd57d45b62e880336fcfef3ef2a7",
  ),
  (
    20250307001101,
    "b5d318f2da5e909d66db190f58b76ae52f6808733d2a88d180555430d28518fee783fb0ff1e1c37ed1dc79627abd9694",
  ),
  (
    20250404160441,
    "962242ef0971de54905cafbdefe26a3c1f32c1c52ef13b6937c2920445b1ce97cd2dcf7db9c293becc95dc55412e7a15",
  ),
];
fn hex_checksum(bytes: &[u8]) -> String {
  bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
