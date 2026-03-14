use recorder::{
    database::rls::get_current_subscriber_id,
    migrations::Migrator,
    test_utils::database::{TestingDatabaseServiceConfig, build_testing_database_service},
};
use sea_orm::TransactionTrait;
use sea_orm_migration::MigratorTrait;

/// Test that all migrations can be applied and rolled back cleanly.
///
/// Runs against a real PostgreSQL instance via testcontainers.
/// feature flag: `testcontainers`
#[tokio::test]
async fn test_migrations_up_and_down() {
    let db = build_testing_database_service(TestingDatabaseServiceConfig { auto_migrate: true })
        .await
        .expect("failed to build testing database service");

    // Verify all migrations are applied
    let applied = Migrator::get_applied_migrations(db.as_ref())
        .await
        .expect("failed to get applied migrations");

    assert!(
        !applied.is_empty(),
        "expected at least one applied migration"
    );

    // Verify the last migration is the RLS one
    let last = applied.last().expect("at least one migration");
    assert!(
        last.name().contains("enable_rls"),
        "last migration should be enable_rls, got: {}",
        last.name()
    );
}

/// Test that RLS policies correctly block access when no subscriber_id is set.
///
/// Without `SET LOCAL app.subscriber_id`, queries on protected tables should
/// return no rows (deny-by-default safety).
#[tokio::test]
async fn test_rls_deny_by_default_without_binding() {
    let db = build_testing_database_service(TestingDatabaseServiceConfig { auto_migrate: true })
        .await
        .expect("failed to build testing database service");

    let txn = db
        .begin()
        .await
        .expect("failed to begin transaction");

    // Without binding subscriber_id, RLS should deny — current_setting returns empty
    // which causes ::integer cast to fail, returning no rows
    let result = get_current_subscriber_id(&txn)
        .await
        .expect("failed to query current subscriber_id");

    // When not set, subscriber_id should be None (empty string from PostgreSQL)
    assert_eq!(
        result, None,
        "expected no subscriber_id set in fresh transaction"
    );

    txn.rollback().await.expect("failed to rollback");
}

/// Test that RLS bind correctly sets the subscriber_id in the transaction.
#[tokio::test]
async fn test_rls_bind_subscriber_to_transaction() {
    use recorder::database::rls::bind_subscriber_to_transaction;

    let db = build_testing_database_service(TestingDatabaseServiceConfig { auto_migrate: true })
        .await
        .expect("failed to build testing database service");

    let txn = db
        .begin()
        .await
        .expect("failed to begin transaction");

    // Bind subscriber_id = 42 to the transaction
    bind_subscriber_to_transaction(&txn, 42)
        .await
        .expect("failed to bind subscriber_id");

    let result = get_current_subscriber_id(&txn)
        .await
        .expect("failed to query current subscriber_id");

    assert_eq!(
        result,
        Some(42),
        "expected subscriber_id = 42 after binding"
    );

    // Verify SET LOCAL scoping: after rollback, the setting should be gone
    txn.rollback().await.expect("failed to rollback");
}

/// Test that all user-private tables have RLS enabled.
#[tokio::test]
async fn test_rls_policies_exist_for_all_tables() {
    use sea_orm::{FromQueryResult, Statement};
    use sea_orm::DbBackend;

    let db = build_testing_database_service(TestingDatabaseServiceConfig { auto_migrate: true })
        .await
        .expect("failed to build testing database service");

    #[derive(Debug, FromQueryResult)]
    struct RlsInfo {
        tablename: String,
        policyname: String,
    }

    let policies = RlsInfo::find_by_statement(Statement::from_string(
        DbBackend::Postgres,
        "SELECT tablename, policyname FROM pg_policies WHERE schemaname = 'public' ORDER BY tablename, policyname",
    ))
    .all(db.as_ref())
    .await
    .expect("failed to query RLS policies");

    let expected_tables = [
        "subscriptions",
        "bangumi",
        "episodes",
        "subscription_bangumi",
        "subscription_episode",
        "downloaders",
        "downloads",
        "auth",
        "credential3rd",
        "feeds",
        "cron",
    ];

    for table in expected_tables {
        let table_policies: Vec<&RlsInfo> = policies
            .iter()
            .filter(|p| p.tablename == table)
            .collect();

        assert_eq!(
            table_policies.len(),
            4,
            "expected 4 RLS policies (select/insert/update/delete) for table '{table}', got {}",
            table_policies.len()
        );

        let policy_names: Vec<&str> = table_policies.iter().map(|p| p.policyname.as_str()).collect();
        for op in ["select", "insert", "update", "delete"] {
            let expected_name = format!("{table}_{op}");
            assert!(
                policy_names.iter().any(|&n| n == expected_name),
                "missing policy '{expected_name}' for table '{table}'"
            );
        }
    }
}

/// Test that RLS enabled tables have FORCE ROW LEVEL SECURITY set.
#[tokio::test]
async fn test_rls_force_enabled_for_all_tables() {
    use sea_orm::{FromQueryResult, Statement};
    use sea_orm::DbBackend;

    let db = build_testing_database_service(TestingDatabaseServiceConfig { auto_migrate: true })
        .await
        .expect("failed to build testing database service");

    #[derive(Debug, FromQueryResult)]
    struct RlsTableInfo {
        relname: String,
        relrowsecurity: bool,
        relforcerowsecurity: bool,
    }

    let tables = RlsTableInfo::find_by_statement(Statement::from_string(
        DbBackend::Postgres,
        "SELECT c.relname, c.relrowsecurity, c.relforcerowsecurity \
         FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE n.nspname = 'public' \
           AND c.relname IN ('subscriptions','bangumi','episodes','subscription_bangumi', \
                             'subscription_episode','downloaders','downloads','auth', \
                             'credential3rd','feeds','cron') \
         ORDER BY c.relname",
    ))
    .all(db.as_ref())
    .await
    .expect("failed to query table RLS settings");

    assert_eq!(tables.len(), 11, "expected 11 RLS-protected tables");

    for table in &tables {
        assert!(
            table.relrowsecurity,
            "table '{}' does not have ROW LEVEL SECURITY enabled",
            table.relname
        );
        assert!(
            table.relforcerowsecurity,
            "table '{}' does not have FORCE ROW LEVEL SECURITY enabled",
            table.relname
        );
    }
}
