//! Real PostgreSQL policies, production GraphQL resolvers and operation
//! ownership.

#[path = "rls/support.rs"]
mod support;

use recorder::{
  database::rls::{bind_subscriber_to_transaction, get_current_subscriber_id},
  models::subscriptions,
};
use sea_orm::{ConnectionTrait, DbBackend, EntityTrait, FromQueryResult, Statement, TransactionTrait};
use support::{RlsFixture, SUBSCRIBER_A, SUBSCRIBER_B};

async fn backend_pid(db: &impl ConnectionTrait) -> i32 {
  db.query_one_raw(Statement::from_string(DbBackend::Postgres, "SELECT pg_backend_pid() AS pid"))
    .await
    .expect("query backend PID")
    .expect("PID row")
    .try_get("", "pid")
    .expect("PID value")
}

async fn insert_subscription(db: &impl ConnectionTrait, subscriber_id: i32, name: &str) -> Result<(), sea_orm::DbErr> {
  db.execute_raw(Statement::from_sql_and_values(
    DbBackend::Postgres,
    "INSERT INTO subscriptions (display_name, subscriber_id, source_url, enabled, category) VALUES ($1, $2, 'https://example.test/new', true, \
     'mikan_subscriber')",
    [name.into(), subscriber_id.into()],
  ))
  .await?;
  Ok(())
}

#[tokio::test]
async fn ordinary_role_subscription_policies_and_transaction_identity() {
  let fixture = RlsFixture::new().await;
  let db = &fixture.app;

  #[derive(Debug, FromQueryResult)]
  struct RoleBoundary {
    role_name: String,
    rolcanlogin: bool,
    rolsuper: bool,
    rolbypassrls: bool,
    rolinherit: bool,
    migration_member: bool,
    table_owner: bool,
    relrowsecurity: bool,
    relforcerowsecurity: bool,
  }
  let role = RoleBoundary::find_by_statement(Statement::from_string(
    DbBackend::Postgres,
    "SELECT current_user::text AS role_name, r.rolcanlogin, r.rolsuper, r.rolbypassrls, r.rolinherit, pg_has_role(current_user, 'migration_owner', 'MEMBER') \
     AS migration_member, c.relowner = r.oid AS table_owner, c.relrowsecurity, c.relforcerowsecurity FROM pg_roles r CROSS JOIN pg_class c WHERE r.rolname = \
     current_user AND c.oid = 'public.subscriptions'::regclass",
  ))
  .one(db)
  .await
  .expect("inspect actual runtime role")
  .expect("runtime role and protected table exist");
  assert_eq!(role.role_name, recorder::database::roles::APP_SCOPED_ACCESS_ROLE);
  assert!(!role.rolcanlogin);
  assert!(!role.rolsuper && !role.rolbypassrls && !role.rolinherit);
  assert!(!role.migration_member && !role.table_owner);
  assert!(role.relrowsecurity && role.relforcerowsecurity);

  assert_eq!(get_current_subscriber_id(db).await.expect("fresh identity getter"), None);
  assert!(
    subscriptions::Entity::find()
      .all(db)
      .await
      .expect("fresh unbound connection denies private rows")
      .is_empty()
  );
  let pid = backend_pid(db).await;

  let txn = db.begin().await.expect("begin A transaction");
  bind_subscriber_to_transaction(&txn, SUBSCRIBER_A).await.expect("bind A");
  assert_eq!(backend_pid(&txn).await, pid);
  let rows = subscriptions::Entity::find().all(&txn).await.expect("query A resources");
  assert_eq!(rows.len(), 1);
  assert_eq!(rows[0].subscriber_id, SUBSCRIBER_A);
  assert_eq!(rows[0].display_name, "A seed");
  let update = txn
    .execute_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "UPDATE subscriptions SET display_name = 'forbidden' WHERE subscriber_id = $1",
      [SUBSCRIBER_B.into()],
    ))
    .await
    .expect("B rows are not visible to UPDATE");
  assert_eq!(update.rows_affected(), 0);
  insert_subscription(&txn, SUBSCRIBER_A, "A committed").await.expect("insert owned resource");
  txn.commit().await.expect("commit owned insert");
  assert_eq!(get_current_subscriber_id(db).await.expect("identity after commit"), None);
  assert_eq!(backend_pid(db).await, pid);

  let txn = db.begin().await.expect("begin illegal insert");
  bind_subscriber_to_transaction(&txn, SUBSCRIBER_A).await.expect("bind A");
  let error = insert_subscription(&txn, SUBSCRIBER_B, "forbidden")
    .await
    .expect_err("A cannot insert B resource");
  eprintln!("Cross-subscriber insert rejected: {error}");
  txn.rollback().await.expect("rollback rejected insert");
  assert_eq!(get_current_subscriber_id(db).await.expect("identity after rejected write"), None);

  let txn = db.begin().await.expect("begin illegal ownership update");
  bind_subscriber_to_transaction(&txn, SUBSCRIBER_A).await.expect("bind A");
  txn
    .execute_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "UPDATE subscriptions SET subscriber_id = $1 WHERE display_name = 'A seed'",
      [SUBSCRIBER_B.into()],
    ))
    .await
    .expect_err("A cannot transfer an owned resource to B");
  txn.rollback().await.expect("rollback rejected ownership update");
  assert_eq!(get_current_subscriber_id(db).await.expect("identity after rejected update"), None);

  let txn = db.begin().await.expect("begin rollback test");
  bind_subscriber_to_transaction(&txn, SUBSCRIBER_A).await.expect("bind A");
  insert_subscription(&txn, SUBSCRIBER_A, "A rolled back").await.expect("insert before rollback");
  assert_eq!(subscriptions::Entity::find().all(&txn).await.expect("own uncommitted rows").len(), 3);
  txn.rollback().await.expect("rollback owned insert");
  assert_eq!(get_current_subscriber_id(db).await.expect("identity after rollback"), None);

  let txn = db.begin().await.expect("verify A commit/rollback");
  bind_subscriber_to_transaction(&txn, SUBSCRIBER_A).await.expect("bind A again");
  let rows = subscriptions::Entity::find().all(&txn).await.expect("read committed A rows");
  assert_eq!(rows.len(), 2);
  assert!(rows.iter().all(|row| row.subscriber_id == SUBSCRIBER_A));
  assert!(rows.iter().any(|row| row.display_name == "A committed"));
  assert!(!rows.iter().any(|row| row.display_name == "A rolled back"));
  txn.commit().await.expect("finish A verification");

  // The existing policy casts an empty setting to integer after SET LOCAL ends.
  // Preserve that diagnostic for R2 without making a database error code an
  // API.
  match subscriptions::Entity::find().all(db).await {
    Ok(rows) => assert!(rows.is_empty(), "unbound reused connection must not disclose rows"),
    Err(error) => eprintln!("R2 diagnostic: unbound reused connection rejected query: {error}"),
  }
  let txn = db.begin().await.expect("reuse connection for B");
  assert_eq!(backend_pid(&txn).await, pid);
  assert_eq!(get_current_subscriber_id(&txn).await.expect("no A identity leaks"), None);
  bind_subscriber_to_transaction(&txn, SUBSCRIBER_B).await.expect("bind B");
  let rows = subscriptions::Entity::find().all(&txn).await.expect("query B resources");
  assert_eq!(rows.len(), 1);
  assert_eq!(rows[0].subscriber_id, SUBSCRIBER_B);
  assert_eq!(rows[0].display_name, "B seed");
  txn.commit().await.expect("finish B transaction");
  assert_eq!(get_current_subscriber_id(db).await.expect("identity after B"), None);
  assert_eq!(backend_pid(db).await, pid);
  fixture.close().await;
}

#[path = "rls/graphql.spec.rs"]
mod graphql;

#[path = "rls/session.spec.rs"]
mod session;

#[path = "rls/protocol.spec.rs"]
mod protocol;

#[path = "rls/migration.spec.rs"]
mod migration;

#[path = "rls/tasks.spec.rs"]
mod tasks;

#[path = "rls/credential_command.spec.rs"]
mod credential_command;

#[path = "rls/cron_permissions.spec.rs"]
mod cron_permissions;

#[path = "rls/task_delivery.spec.rs"]
mod task_delivery;

#[path = "rls/task_delivery_lifecycle.spec.rs"]
mod task_delivery_lifecycle;

#[path = "rls/media_delivery.spec.rs"]
mod media_delivery;
