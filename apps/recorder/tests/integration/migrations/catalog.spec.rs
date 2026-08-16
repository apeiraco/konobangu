use recorder::{
  database::rls::get_current_subscriber_id,
  migrations::Migrator,
  test_utils::database::{TestingDatabaseServiceConfig, build_testing_database_service},
};
use sea_orm::TransactionTrait;
use sea_orm_migration::MigratorTrait;

// Catalog checks use the migration owner. Ordinary-role behavior is covered by
// the RLS integration target (NOSUPERUSER, NOBYPASSRLS), including irreversible
// downgrade rejection.
#[tokio::test]
async fn test_migrations_up_and_reentry() {
  let db = build_testing_database_service(TestingDatabaseServiceConfig { auto_migrate: true })
    .await
    .unwrap();
  let expected: Vec<_> = Migrator::migrations().iter().map(|m| m.name().to_owned()).collect();
  let applied = Migrator::get_applied_migrations(db.as_ref()).await.unwrap();
  let names: Vec<_> = applied.iter().map(|m| m.name().to_owned()).collect();
  assert_eq!(names, expected);
  Migrator::up(db.as_ref(), None).await.unwrap();
  let repeated = Migrator::get_applied_migrations(db.as_ref()).await.unwrap();
  assert_eq!(repeated.iter().map(|m| m.name().to_owned()).collect::<Vec<_>>(), names);
}

#[tokio::test]
async fn test_subscriber_setting_is_unbound_in_fresh_transaction() {
  let db = build_testing_database_service(TestingDatabaseServiceConfig { auto_migrate: true })
    .await
    .unwrap();
  let txn = db.begin().await.unwrap();
  assert_eq!(get_current_subscriber_id(&txn).await.unwrap(), None);
  txn.rollback().await.unwrap();
}

/// Test that RLS bind correctly sets the subscriber_id in the transaction.
#[tokio::test]
async fn test_rls_bind_subscriber_to_transaction() {
  use recorder::database::rls::bind_subscriber_to_transaction;

  let db = build_testing_database_service(TestingDatabaseServiceConfig { auto_migrate: true })
    .await
    .expect("failed to build testing database service");

  let txn = db.begin().await.expect("failed to begin transaction");

  // Bind subscriber_id = 42 to the transaction
  bind_subscriber_to_transaction(&txn, 42).await.expect("failed to bind subscriber_id");

  let result = get_current_subscriber_id(&txn).await.expect("failed to query current subscriber_id");

  assert_eq!(result, Some(42), "expected subscriber_id = 42 after binding");

  // Verify SET LOCAL scoping: after rollback, the setting should be gone
  txn.rollback().await.expect("failed to rollback");
}

const BUSINESS_TABLES: [&str; 10] = [
  "subscriptions",
  "bangumi",
  "episodes",
  "subscription_bangumi",
  "subscription_episode",
  "downloaders",
  "downloads",
  "credential3rd",
  "feeds",
  "cron",
];

#[tokio::test]
async fn test_current_rls_catalog_contract() {
  use sea_orm::{DbBackend, FromQueryResult, Statement};
  let db = build_testing_database_service(TestingDatabaseServiceConfig { auto_migrate: true })
    .await
    .unwrap();
  #[derive(Debug, FromQueryResult)]
  struct Policy {
    tablename: String,
    policyname: String,
    cmd: String,
    roles: Vec<String>,
  }
  let policies = Policy::find_by_statement(Statement::from_string(
    DbBackend::Postgres,
    "SELECT tablename, policyname, cmd, roles::text[] AS roles FROM pg_policies WHERE schemaname = 'public' ORDER BY tablename, policyname",
  ))
  .all(db.as_ref())
  .await
  .unwrap();
  for table in BUSINESS_TABLES {
    let found: Vec<_> = policies.iter().filter(|p| p.tablename == table).collect();
    assert_eq!(found.len(), if table == "cron" { 5 } else { 4 }, "{table}");
    for (op, cmd) in [("select", "SELECT"), ("insert", "INSERT"), ("update", "UPDATE"), ("delete", "DELETE")] {
      let policy = found
        .iter()
        .find(|p| p.policyname == format!("{table}_{op}"))
        .expect("missing business CRUD policy");
      assert_eq!(policy.cmd, cmd);
      assert_eq!(policy.roles, ["public"]);
    }
  }
  for table in ["cron", "task_runs", "task_outbox"] {
    let policy = policies
      .iter()
      .find(|p| p.policyname == format!("{table}_task_control"))
      .expect("missing task-control policy");
    assert_eq!(policy.roles, ["konobangu_task_control_access"]);
    assert_eq!(policy.cmd, "ALL");
  }
  for table in ["task_runs", "task_outbox"] {
    let found: Vec<_> = policies.iter().filter(|p| p.tablename == table).collect();
    assert_eq!(found.len(), 2);
    let owner = found.iter().find(|p| p.policyname == format!("{table}_app_scoped")).unwrap();
    assert_eq!(owner.roles, ["public"]);
    assert_eq!(owner.cmd, "ALL");
  }
  #[derive(Debug, FromQueryResult)]
  struct Table {
    schema: String,
    relname: String,
    relrowsecurity: bool,
    relforcerowsecurity: bool,
  }
  let tables = Table::find_by_statement(Statement::from_string(
    DbBackend::Postgres,
    "SELECT n.nspname AS schema, c.relname, c.relrowsecurity, c.relforcerowsecurity FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE \
     n.nspname IN ('public', 'auth_identity') AND c.relkind = 'r'",
  ))
  .all(db.as_ref())
  .await
  .unwrap();
  for name in BUSINESS_TABLES.into_iter().chain(["task_runs", "task_outbox"]) {
    let table = tables
      .iter()
      .find(|t| t.schema == "public" && t.relname == name)
      .expect("missing protected table");
    assert!(table.relrowsecurity && table.relforcerowsecurity, "{name}");
  }
  assert!(!tables.iter().any(|t| t.schema == "public" && t.relname == "auth"));
  let identity = tables
    .iter()
    .find(|t| t.schema == "auth_identity" && t.relname == "auth")
    .expect("missing identity mapping");
  assert!(!identity.relrowsecurity && !identity.relforcerowsecurity);
  #[derive(Debug, FromQueryResult)]
  struct Role {
    rolsuper: bool,
    rolbypassrls: bool,
    rolcanlogin: bool,
  }
  let role = Role::find_by_statement(Statement::from_string(
    DbBackend::Postgres,
    "SELECT rolsuper, rolbypassrls, rolcanlogin FROM pg_roles WHERE rolname = 'konobangu_task_control_access'",
  ))
  .one(db.as_ref())
  .await
  .unwrap()
  .expect("missing task-control role");
  assert!(!role.rolsuper && !role.rolbypassrls && !role.rolcanlogin);
}
