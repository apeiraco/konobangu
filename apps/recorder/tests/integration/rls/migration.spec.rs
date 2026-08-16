use recorder::{
  database::DatabaseService,
  migrations::{
    Migrator, MigratorTrait,
    access::{RuntimeLogins, grant_runtime_memberships},
  },
};
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};

use super::support::RlsFixture;

#[tokio::test]
async fn configured_login_capabilities_enforce_current_access_boundaries() {
  use std::sync::Arc;

  use recorder::{
    app::AppContextTrait,
    auth::{config::AuthSessionConfig, session::runtime::SessionRuntime},
    database::rls::bind_subscriber_to_transaction,
    task::{TaskConfig, TaskService},
    test_utils::app::TestingAppContext,
  };

  let fixture = RlsFixture::new().await;
  let roles = RuntimeLogins {
    app_scoped: "app scoped \"测试".into(),
    auth: "app scoped \"测试".into(),
    task_control: Some("app scoped \"测试".into()),
  };
  fixture
    .bootstrap
    .execute_unprepared("CREATE ROLE \"app scoped \"\"测试\" LOGIN NOSUPERUSER NOBYPASSRLS")
    .await
    .unwrap();
  let uri = |role: &str| {
    let mut url = url::Url::parse(&fixture.bootstrap.config.uri).unwrap();
    url.set_username(role).unwrap();
    url.set_password(Some("role-test-password")).unwrap();
    url.to_string()
  };
  let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("../../temp/verification/fixtures")
    .join(format!("role-cli-{}", uuid::Uuid::new_v4()));
  std::fs::create_dir_all(&directory).unwrap();
  let config_path = directory.join("config.json");
  let dotenv_path = directory.join("fixture.env");
  std::fs::write(&dotenv_path, "").unwrap();
  std::fs::write(&config_path, serde_json::to_vec(&serde_json::json!({
    "database": {"url": uri(&roles.app_scoped), "migration": {"url": fixture.bootstrap.config.uri, "auto_run": false}},
    "auth": {"provider": {"type": "basic", "username": "fixture", "password": "fixture"}, "session": {"database_url": uri(&roles.auth), "public_url": "http://127.0.0.1:5001", "cookie_secure": false}},
    "scheduler": {"database": {"url": uri(roles.task_control.as_deref().unwrap())}},
  })).unwrap()).unwrap();
  let cli = std::process::Command::new(env!("CARGO_BIN_EXE_migrate-up"))
    .env_clear()
    .args(["--environment", "testing", "--config-file"])
    .arg(&config_path)
    .arg("--dotenv-file")
    .arg(&dotenv_path)
    .output()
    .unwrap();
  std::fs::remove_dir_all(&directory).unwrap();
  assert!(cli.status.success(), "role provisioning CLI: {}", String::from_utf8_lossy(&cli.stderr));
  // Reentry reapplies grants without resetting independently provisioned
  // passwords.
  for role in [&roles.app_scoped] {
    let password_sql: String = fixture
      .bootstrap
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT format('ALTER ROLE %I PASSWORD %L', $1::text, $2::text) AS sql",
        [role.clone().into(), "role-test-password".into()],
      ))
      .await
      .unwrap()
      .unwrap()
      .try_get("", "sql")
      .unwrap();
    fixture.bootstrap.execute_unprepared(&password_sql).await.unwrap();
  }
  grant_runtime_memberships(fixture.bootstrap.as_ref(), &roles).await.unwrap();
  let mut scoped = fixture.bootstrap.config.clone();
  scoped.auto_migrate = true;
  scoped.migration_database_uri = Some(fixture.bootstrap.config.uri.clone());
  scoped.uri = uri(&roles.app_scoped);
  let ctx = Arc::new(TestingAppContext::builder().db(DatabaseService::from_config(scoped).await.unwrap()).build());
  assert!(
    ctx
      .db()
      .query_all_raw(Statement::from_string(DbBackend::Postgres, "SELECT id FROM subscriptions"))
      .await
      .unwrap()
      .is_empty()
  );
  let txn = ctx.db().begin().await.unwrap();
  bind_subscriber_to_transaction(&txn, super::support::SUBSCRIBER_A).await.unwrap();
  assert_eq!(
    txn
      .query_all_raw(Statement::from_string(DbBackend::Postgres, "SELECT id FROM subscriptions"))
      .await
      .unwrap()
      .len(),
    1
  );
  txn.rollback().await.unwrap();
  let auth_config: AuthSessionConfig = serde_json::from_value(serde_json::json!({
    "identity_database_uri": uri(&roles.auth),
    "external_base_url": "http://127.0.0.1:5001",
    "session_cookie_secure": false,
  }))
  .unwrap();
  let auth = SessionRuntime::new(auth_config).await.unwrap();
  assert!(auth.identity_db.execute_unprepared("SELECT id FROM task_runs").await.is_err());
  let config = TaskConfig {
    queue_database_uri: Some(uri(roles.task_control.as_deref().unwrap())),
    ..Default::default()
  };
  let task = TaskService::from_config_and_ctx(config, ctx.clone()).await.unwrap();
  task
    .queue_database()
    .unwrap()
    .execute_unprepared("SELECT id FROM task_runs; SELECT id FROM cron; SELECT id FROM apalis.jobs")
    .await
    .unwrap();
  assert!(task.queue_database().unwrap().execute_unprepared("SELECT id FROM subscriptions").await.is_err());
  assert!(
    task
      .queue_database()
      .unwrap()
      .execute_unprepared("SELECT id FROM auth_identity.auth")
      .await
      .is_err()
  );
  task.close().await.unwrap();
  auth.close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
}

#[tokio::test]
async fn legacy_identity_migration_requires_attribution_and_preserves_shared_owners() {
  let fixture = RlsFixture::new().await;
  fixture.bootstrap.execute_unprepared("CREATE DATABASE legacy_test").await.unwrap();
  let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("../../temp/verification/fixtures")
    .join(format!("migration-cli-{}", uuid::Uuid::new_v4()));
  std::fs::create_dir_all(&directory).unwrap();
  let config_path = directory.join("config.json");
  let dotenv_path = directory.join("fixture.env");
  std::fs::write(&dotenv_path, "").unwrap();
  std::fs::write(&config_path, serde_json::to_vec(&serde_json::json!({
    "database": fixture.bootstrap.config,
    "auth": { "provider": {"type": "basic", "username": "fixture", "password": "fixture"}, "session": {"database_url": fixture.identity_uri, "public_url": "http://127.0.0.1:5001", "cookie_secure": false} },
    "server": { "host": "127.0.0.1" }, "logger": { "enable": false, "level": "off", "format": "compact" }
  })).unwrap()).unwrap();
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600)).unwrap();
  }
  let cli = std::process::Command::new(env!("CARGO_BIN_EXE_migrate-up"))
    .env_clear()
    .args(["--environment", "production", "--config-file"])
    .arg(&config_path)
    .arg("--dotenv-file")
    .arg(&dotenv_path)
    .output()
    .unwrap();
  std::fs::remove_dir_all(&directory).unwrap();
  assert!(cli.status.success(), "documented owner migration CLI must succeed: {:?}", cli.status);
  let mut config = fixture.bootstrap.config.clone();
  config.uri = config.uri.replace("/rls_test", "/legacy_test");
  config.auto_migrate = false;
  let legacy = DatabaseService::from_migration_config(config.clone()).await.unwrap();
  legacy.execute_unprepared("CREATE SCHEMA apalis").await.unwrap();
  let report = recorder::migrations::legacy::preflight(legacy.as_ref()).await.unwrap();
  assert!(report.issues.iter().any(|issue| issue.contains("Unrecognized apalis schema")));
  assert!(legacy.migrate_up().await.is_err(), "an unknown queue schema must reject before bootstrap DDL");
  legacy.execute_unprepared("DROP SCHEMA apalis").await.unwrap();
  recorder::migrations::legacy::bootstrap(legacy.as_ref()).await.unwrap();
  legacy
    .execute_unprepared(
      "CREATE TABLE public._sqlx_migrations (version bigint PRIMARY KEY, description text NOT NULL, installed_on timestamptz NOT NULL DEFAULT now(), success \
       boolean NOT NULL, checksum bytea NOT NULL, execution_time bigint NOT NULL)",
    )
    .await
    .unwrap();

  // Published SQLx history is accepted unchanged; corruption fails before DDL.
  legacy
    .execute_unprepared(
      "INSERT INTO public._sqlx_migrations(version,description,success,checksum,execution_time) VALUES (20220530084123,'jobs \
       workers',true,decode('0e2ae5fd117c32925d7ff4ba4f26368269fba7706bf1c3e9578a763a3e2acc5fd44c8edf2aa25e7c0ac1c6ebc8e95a9a','hex'),0)",
    )
    .await
    .unwrap();
  assert!(recorder::migrations::legacy::preflight(legacy.as_ref()).await.unwrap().issues.is_empty());
  legacy
    .execute_unprepared("UPDATE public._sqlx_migrations SET checksum=decode('01','hex') WHERE version=20220530084123")
    .await
    .unwrap();
  assert!(legacy.migrate_up().await.is_err());
  legacy
    .execute_unprepared(
      "UPDATE public._sqlx_migrations SET \
       checksum=decode('0e2ae5fd117c32925d7ff4ba4f26368269fba7706bf1c3e9578a763a3e2acc5fd44c8edf2aa25e7c0ac1c6ebc8e95a9a','hex') WHERE version=20220530084123",
    )
    .await
    .unwrap();
  legacy.execute_unprepared("ALTER TABLE apalis.workers RENAME TO workers_hidden").await.unwrap();
  let report = recorder::migrations::legacy::preflight(legacy.as_ref()).await.unwrap();
  assert!(report.issues.iter().any(|issue| issue.contains("missing workers")));
  assert!(legacy.migrate_up().await.is_err(), "unverifiable liveness must reject before application DDL");
  legacy.execute_unprepared("ALTER TABLE apalis.workers_hidden RENAME TO workers").await.unwrap();
  Migrator::up(
    legacy.as_ref(),
    Some(
      Migrator::migrations()
        .iter()
        .position(|migration| migration.name() == "m20261003_000001_identity_session")
        .unwrap() as u32,
    ),
  )
  .await
  .unwrap();
  legacy.execute_unprepared("INSERT INTO auth(pid, auth_type, subscriber_id) VALUES ('existing-subject', 'oidc', 1), ('shared-subject', 'oidc', 1); INSERT INTO subscriptions(display_name, subscriber_id, source_url, enabled, category) VALUES ('legacy subscription', 1, 'https://example.test/legacy', true, 'mikan_subscriber')").await.unwrap();
  assert!(
    legacy.migrate_up().await.is_err(),
    "unknown issuer attribution must stop before changing identity data"
  );
  let untouched = legacy
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT count(*) FROM public.auth WHERE auth_type = 'oidc'",
    ))
    .await
    .unwrap()
    .unwrap();
  assert_eq!(untouched.try_get_by_index::<i64>(0).unwrap(), 2);
  let no_schema = legacy
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT EXISTS(SELECT 1 FROM pg_namespace WHERE nspname = 'auth_identity')",
    ))
    .await
    .unwrap()
    .unwrap();
  assert!(!no_schema.try_get_by_index::<bool>(0).unwrap());
  // Populate the published 0.7 schema after real historical application DDL.
  for (index, status) in ["Pending", "Scheduled", "Running", "Done", "Failed", "Killed"].into_iter().enumerate() {
    legacy
      .execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO apalis.jobs(id,job,job_type,status,attempts,max_attempts,run_at,done_at,last_error) VALUES \
         ($1,$2,'system_task',$3,2,7,'2030-01-01T00:00:00Z',CASE WHEN $3 IN ('Done','Failed','Killed') THEN '2029-12-31T00:00:00Z'::timestamptz ELSE NULL \
         END,'legacy safe error')",
        [
          format!("legacy-{index}").into(),
          serde_json::json!({"task_type":"test","task_id":"legacy-echo","subscriber_id":1}).into(),
          status.into(),
        ],
      ))
      .await
      .unwrap();
  }
  legacy.execute_unprepared(r#"UPDATE apalis.jobs SET job_type='subscriber_task',job='{"task_type":"sync_one_subscription_sources","subscriber_id":1,"subscription_id":1}' WHERE id='legacy-3'"#).await.unwrap();
  let report = recorder::migrations::legacy::preflight(legacy.as_ref()).await.unwrap();
  assert_eq!(report.legacy_tasks, 6);
  assert!(report.issues.is_empty(), "{:?}", report.issues);
  legacy
    .execute_unprepared("UPDATE apalis.jobs SET status='Unknown' WHERE id='legacy-0'")
    .await
    .unwrap();
  assert!(legacy.migrate_up().await.is_err());
  assert!(
    !legacy
      .query_one_raw(Statement::from_string(DbBackend::Postgres, "SELECT to_regclass('task_runs') IS NOT NULL"))
      .await
      .unwrap()
      .unwrap()
      .try_get_by_index::<bool>(0)
      .unwrap()
  );
  legacy
    .execute_unprepared("UPDATE apalis.jobs SET status='Pending' WHERE id='legacy-0'; UPDATE apalis.jobs SET job_type='unknown_kind' WHERE id='legacy-1'")
    .await
    .unwrap();
  assert!(!recorder::migrations::legacy::preflight(legacy.as_ref()).await.unwrap().issues.is_empty());
  legacy
    .execute_unprepared(
      "UPDATE apalis.jobs SET job_type='system_task' WHERE id='legacy-1'; INSERT INTO \
       public._sqlx_migrations(version,description,installed_on,success,checksum,execution_time) \
       VALUES(999999,'foreign-component',now(),true,decode('01','hex'),0)",
    )
    .await
    .unwrap();
  assert!(legacy.migrate_up().await.is_err());
  legacy
    .execute_unprepared(
      "DELETE FROM public._sqlx_migrations WHERE version=999999; INSERT INTO apalis.workers(id,worker_type,storage_name) VALUES \
       ('live-fixture','system_task','postgres')",
    )
    .await
    .unwrap();
  assert!(legacy.migrate_up().await.is_err());
  legacy.execute_unprepared("DELETE FROM apalis.workers WHERE id='live-fixture'").await.unwrap();
  // Exercise a real owner backup/restore entirely inside the disposable
  // cluster.
  let dump = std::process::Command::new("docker")
    .args(["exec", fixture.container_id(), "pg_dump", "-U", "migration_owner", "-Fc", "legacy_test"])
    .output()
    .unwrap();
  assert!(dump.status.success());
  let backup_directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../temp/verification/fixtures/backups");
  std::fs::create_dir_all(&backup_directory).unwrap();
  std::fs::write(backup_directory.join(format!("{}.dump", uuid::Uuid::now_v7())), &dump.stdout).unwrap();
  fixture.bootstrap.execute_unprepared("CREATE DATABASE restore_test").await.unwrap();
  let mut restore = std::process::Command::new("docker")
    .args([
      "exec",
      "-i",
      fixture.container_id(),
      "pg_restore",
      "-U",
      "migration_owner",
      "-d",
      "restore_test",
    ])
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .spawn()
    .unwrap();
  use std::io::Write;
  restore.stdin.take().unwrap().write_all(&dump.stdout).unwrap();
  assert!(restore.wait().unwrap().success());
  let mut restored_config = config.clone();
  restored_config.uri = restored_config.uri.replace("/legacy_test", "/restore_test");
  let restored = DatabaseService::from_migration_config(restored_config).await.unwrap();
  assert_eq!(recorder::migrations::legacy::preflight(restored.as_ref()).await.unwrap().legacy_tasks, 6);
  restored.as_ref().clone().close().await.unwrap();
  config.legacy_oidc_issuer = Some("https://legacy-issuer.example".into());
  let attributed = DatabaseService::from_migration_config(config).await.unwrap();
  attributed.migrate_up().await.unwrap();
  attributed.migrate_up().await.unwrap();
  let imported = attributed
    .query_all_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT id,status,subscriber_id,attempts,max_attempts,run_at,done_at,recovering,legacy_execution FROM task_runs ORDER BY id",
    ))
    .await
    .unwrap();
  assert_eq!(imported.len(), 6);
  for (index, row) in imported.iter().enumerate() {
    assert_eq!(row.try_get::<String>("", "id").unwrap(), format!("legacy-{index}"));
    assert_eq!(row.try_get::<i32>("", "subscriber_id").unwrap(), 1);
    assert_eq!(row.try_get::<i32>("", "attempts").unwrap(), 2);
    assert_eq!(row.try_get::<i32>("", "max_attempts").unwrap(), 7);
    assert_eq!(row.try_get::<bool>("", "recovering").unwrap(), index == 2);
    assert_eq!(
      row.try_get::<String>("", "status").unwrap(),
      ["Pending", "Scheduled", "Pending", "Done", "Failed", "Killed"][index]
    );
  }
  assert_eq!(
    attributed
      .query_one_raw(Statement::from_string(DbBackend::Postgres, "SELECT count(*) FROM task_outbox"))
      .await
      .unwrap()
      .unwrap()
      .try_get_by_index::<i64>(0)
      .unwrap(),
    3
  );
  assert_eq!(
    attributed
      .query_one_raw(Statement::from_string(DbBackend::Postgres, "SELECT count(*) FROM apalis_legacy_v07.jobs"))
      .await
      .unwrap()
      .unwrap()
      .try_get_by_index::<i64>(0)
      .unwrap(),
    6
  );
  assert_eq!(
    attributed
      .query_one_raw(Statement::from_string(DbBackend::Postgres, "SELECT count(*) FROM apalis.jobs"))
      .await
      .unwrap()
      .unwrap()
      .try_get_by_index::<i64>(0)
      .unwrap(),
    0
  );
  // Fail a later queue initialization stage, retain the committed application
  // cutover, then retry from that stopped state without reimporting data.
  attributed
    .execute_unprepared("DROP SCHEMA apalis CASCADE; CREATE SCHEMA apalis; CREATE TABLE apalis.jobs (bad integer)")
    .await
    .unwrap();
  assert!(attributed.migrate_up().await.is_err());
  attributed.execute_unprepared("DROP SCHEMA apalis CASCADE").await.unwrap();
  attributed.migrate_up().await.unwrap();
  let rows = attributed
    .query_all_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT issuer, subscriber_id FROM auth_identity.auth WHERE auth_type = 'oidc' ORDER BY pid",
    ))
    .await
    .unwrap();
  assert_eq!(rows.len(), 2);
  for row in rows {
    assert_eq!(row.try_get::<String>("", "issuer").unwrap(), "https://legacy-issuer.example");
    assert_eq!(row.try_get::<i32>("", "subscriber_id").unwrap(), 1);
  }
  let basic = attributed
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT issuer IS NULL FROM auth_identity.auth WHERE auth_type = 'basic'",
    ))
    .await
    .unwrap()
    .unwrap();
  assert!(basic.try_get_by_index::<bool>(0).unwrap());
  let subscription = attributed
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT subscriber_id FROM subscriptions WHERE display_name = 'legacy subscription'",
    ))
    .await
    .unwrap()
    .unwrap();
  assert_eq!(subscription.try_get_by_index::<i32>(0).unwrap(), 1);
  let transaction = attributed.begin().await.unwrap();
  recorder::database::rls::bind_subscriber_to_transaction(&transaction, 1).await.unwrap();
  let filter = sea_orm::Condition::all().add(sea_orm::ColumnTrait::eq(&recorder::models::subscriptions::Column::Id, 1));
  assert_eq!(recorder::models::subscriptions::operation::delete_owned(&transaction, filter).await.unwrap(), 1);
  transaction.commit().await.unwrap();
  let history = attributed
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT subscription_id IS NULL AS detached,payload->>'subscription_id' AS original FROM task_runs WHERE id='legacy-3'",
    ))
    .await
    .unwrap()
    .unwrap();
  assert!(history.try_get::<bool>("", "detached").unwrap());
  assert_eq!(history.try_get::<String>("", "original").unwrap(), "1");
  attributed.migrate_up().await.unwrap();
  assert!(
    Migrator::down(attributed.as_ref(), Some(1)).await.is_err(),
    "unsafe down must require backup restoration"
  );
  legacy.as_ref().clone().close().await.unwrap();
  attributed.as_ref().clone().close().await.unwrap();
  fixture.close().await;
}

#[tokio::test]
async fn single_createrole_owner_bootstraps_restricted_pools_without_manual_provisioning() {
  use std::sync::Arc;

  use figment::{Figment, providers::Serialized};
  use recorder::{
    app::AppConfig,
    auth::session::runtime::SessionRuntime,
    database::roles::{APP_SCOPED_ACCESS_ROLE, AUTH_ACCESS_ROLE, CAPABILITY_ROLES, TASK_CONTROL_ACCESS_ROLE, set_local_role},
    task::TaskService,
    test_utils::app::TestingAppContext,
  };
  use testcontainers::{ImageExt, runners::AsyncRunner};
  use testcontainers_modules::postgres::Postgres;

  let container = Postgres::default().with_tag("18-alpine").start().await.unwrap();
  let host = container.get_host().await.unwrap();
  let port = container.get_host_port_ipv4(5432).await.unwrap();
  let bootstrap = sea_orm::Database::connect(format!("postgres://postgres:postgres@{host}:{port}/postgres"))
    .await
    .unwrap();
  bootstrap
    .execute_unprepared("CREATE ROLE deployment_owner LOGIN CREATEROLE NOSUPERUSER NOBYPASSRLS PASSWORD 'deployment-password'")
    .await
    .unwrap();
  bootstrap
    .execute_unprepared("CREATE DATABASE deployment_test OWNER deployment_owner")
    .await
    .unwrap();
  let uri = format!("postgres://deployment_owner:deployment-password@{host}:{port}/deployment_test");
  let mut config: AppConfig = Figment::new()
    .merge(AppConfig::default_provider())
    .merge(Serialized::defaults(serde_json::json!({
      "database": {"url": uri, "pool": {"min_connections": 1, "max_connections": 1}},
      "auth": {"provider": {"type": "basic", "username": "fixture", "password": "fixture"},
               "session": {"public_url": "http://127.0.0.1:5001", "cookie_secure": false}},
    })))
    .extract()
    .unwrap();
  config.resolve_database_urls();
  assert!(config.database.auto_migrate);
  assert_eq!(config.auth.session().identity_database_uri, config.database.uri);
  assert_eq!(config.task.queue_database_uri.as_deref(), Some(config.database.uri.as_str()));
  let db = DatabaseService::from_app_config(&config).await.unwrap();
  for capability in CAPABILITY_ROLES {
    let row = db
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT rolcanlogin, rolsuper, rolbypassrls, pg_has_role(session_user, oid, 'USAGE') AS inherited, pg_has_role(session_user, oid, 'SET') AS \
         switchable FROM pg_roles WHERE rolname=$1",
        [capability.into()],
      ))
      .await
      .unwrap()
      .unwrap();
    assert!(!row.try_get::<bool>("", "rolcanlogin").unwrap());
    assert!(!row.try_get::<bool>("", "rolsuper").unwrap());
    assert!(!row.try_get::<bool>("", "rolbypassrls").unwrap());
    assert!(!row.try_get::<bool>("", "inherited").unwrap());
    assert!(row.try_get::<bool>("", "switchable").unwrap());
  }
  async fn role(db: &impl ConnectionTrait) -> (String, i32) {
    let row = db
      .query_one_raw(Statement::from_string(
        DbBackend::Postgres,
        "SELECT current_user::text AS role, pg_backend_pid() AS pid",
      ))
      .await
      .unwrap()
      .unwrap();
    (row.try_get("", "role").unwrap(), row.try_get("", "pid").unwrap())
  }
  let baseline = role(&db).await;
  assert_eq!(baseline.0, APP_SCOPED_ACCESS_ROLE);
  for commit in [true, false] {
    let txn = db.begin().await.unwrap();
    set_local_role(&txn, AUTH_ACCESS_ROLE).await.unwrap();
    assert_eq!(role(&txn).await, (AUTH_ACCESS_ROLE.to_string(), baseline.1));
    if commit {
      txn.commit().await.unwrap();
    } else {
      txn.rollback().await.unwrap();
    }
    assert_eq!(role(&db).await, baseline);
  }
  // A new app startup reruns idempotent migrations and grants using the same
  // non-superuser owner credential.
  let restarted = DatabaseService::from_app_config(&config).await.unwrap();
  assert_eq!(role(&restarted).await.0, APP_SCOPED_ACCESS_ROLE);
  restarted.as_ref().clone().close().await.unwrap();
  let auth = SessionRuntime::new(config.auth.session().clone()).await.unwrap();
  assert_eq!(role(&auth.identity_db).await.0, AUTH_ACCESS_ROLE);
  let stored_role: String = sqlx08::query_scalar("SELECT current_user::text").fetch_one(&auth.session_pool).await.unwrap();
  assert_eq!(stored_role, AUTH_ACCESS_ROLE);
  assert!(auth.identity_db.execute_unprepared("SELECT id FROM subscriptions").await.is_err());
  let ctx = Arc::new(TestingAppContext::builder().db(db).build());
  let task = TaskService::from_config_and_ctx(config.task, ctx.clone()).await.unwrap();
  assert_eq!(role(task.queue_database().unwrap()).await.0, TASK_CONTROL_ACCESS_ROLE);
  assert!(
    task
      .queue_database()
      .unwrap()
      .execute_unprepared("SELECT id FROM auth_identity.auth")
      .await
      .is_err()
  );
  task.close().await.unwrap();
  auth.close().await.unwrap();
  use recorder::app::AppContextTrait;
  ctx.db().as_ref().clone().close().await.unwrap();
  bootstrap.close().await.unwrap();
  container.rm().await.unwrap();
}
