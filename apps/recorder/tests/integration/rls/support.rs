use recorder::{
  database::{DatabaseConfig, DatabaseService},
  migrations::access::{RuntimeLogins, grant_runtime_memberships},
};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

#[allow(
  dead_code,
  reason = "Shared RLS tests and native smoke receipts use this fixture metadata; the HTTP support server does not"
)]
pub const SUBSCRIBER_A: i32 = 1001;
#[allow(
  dead_code,
  reason = "Shared RLS tests and native smoke receipts use this fixture metadata; the HTTP support server does not"
)]
pub const SUBSCRIBER_B: i32 = 1002;

// Each fixture owns its container. No development URL or container pruning is
// used.
pub struct RlsFixture {
  pub app: DatabaseConnection,
  pub bootstrap: DatabaseService,
  pub app_uri: String,
  pub identity_uri: String,
  pub queue_uri: String,
  _container: ContainerAsync<Postgres>,
}

impl RlsFixture {
  #[allow(
    dead_code,
    reason = "Shared RLS tests and native smoke receipts use this fixture metadata; the HTTP support server does not"
  )]
  pub fn container_id(&self) -> &str {
    self._container.id()
  }
  pub async fn new() -> Self {
    let container = Postgres::default()
      .with_db_name("rls_test")
      .with_user("migration_owner")
      .with_password("migration_test_password")
      .with_tag("18-alpine")
      .start()
      .await
      .expect("PostgreSQL testcontainer must start");
    let host = container.get_host().await.expect("container host");
    let port = container.get_host_port_ipv4(5432).await.expect("PostgreSQL port");
    let bootstrap = DatabaseService::from_migration_config(DatabaseConfig {
      uri: format!("postgres://migration_owner:migration_test_password@{host}:{port}/rls_test"),
      enable_logging: false,
      min_connections: 1,
      max_connections: 1,
      connect_timeout: 10000,
      idle_timeout: 60000,
      acquire_timeout: Some(10000),
      auto_migrate: true,
      legacy_oidc_issuer: None,
      migration_database_uri: None,
    })
    .await
    .expect("existing Apalis and application migrations must apply");
    bootstrap
      .execute_unprepared(
        "CREATE ROLE rls_app LOGIN NOSUPERUSER NOBYPASSRLS PASSWORD 'rls_test_password';
       CREATE ROLE auth_app LOGIN NOSUPERUSER NOBYPASSRLS PASSWORD 'identity_test_password';
       CREATE ROLE task_control_app LOGIN NOSUPERUSER NOBYPASSRLS PASSWORD 'queue_test_password'",
      )
      .await
      .expect("fixture-owned login credentials");
    grant_runtime_memberships(
      bootstrap.as_ref(),
      &RuntimeLogins {
        app_scoped: "rls_app".into(),
        auth: "auth_app".into(),
        task_control: Some("task_control_app".into()),
      },
    )
    .await
    .expect("provision production access contracts for fixture logins");
    bootstrap
      .execute_unprepared(
        "INSERT INTO subscribers (id, display_name) VALUES (1001, 'Subscriber A'), (1002, 'Subscriber B');
       INSERT INTO subscriptions (display_name, subscriber_id, \
         source_url, enabled, category) VALUES
         ('A seed', 1001, 'https://example.test/a', true, 'mikan_subscriber'),
         ('B seed', 1002, 'https://example.test/b', \
         true, 'mikan_subscriber');",
      )
      .await
      .expect("fixture passwords and two subscribers' resources");
    let mut options = ConnectOptions::new(format!("postgres://rls_app:rls_test_password@{host}:{port}/rls_test"));
    // A single physical connection makes identity reuse deterministic.
    options.min_connections(1).max_connections(1).sqlx_logging(false);
    recorder::database::roles::restrict_pool(&mut options, recorder::database::roles::APP_SCOPED_ACCESS_ROLE);
    let app = Database::connect(options).await.expect("ordinary application role connects");
    Self {
      app,
      bootstrap,
      app_uri: format!("postgres://rls_app:rls_test_password@{host}:{port}/rls_test"),
      queue_uri: format!("postgres://task_control_app:queue_test_password@{host}:{port}/rls_test"),
      identity_uri: format!("postgres://auth_app:identity_test_password@{host}:{port}/rls_test"),
      _container: container,
    }
  }

  pub async fn close(self) {
    self.app.close().await.expect("close application pool");
    self.bootstrap.as_ref().clone().close().await.expect("close bootstrap pool");
    self._container.rm().await.expect("remove owned PostgreSQL fixture");
  }
}
