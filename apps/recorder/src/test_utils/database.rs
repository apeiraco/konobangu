use crate::{
  database::{DatabaseConfig, DatabaseService},
  errors::RecorderResult,
};

#[derive(Clone, Debug)]
pub struct TestingDatabaseServiceConfig {
  pub auto_migrate: bool,
}

impl Default for TestingDatabaseServiceConfig {
  fn default() -> Self {
    Self { auto_migrate: true }
  }
}

#[cfg(feature = "testcontainers")]
pub async fn build_testing_database_service(config: TestingDatabaseServiceConfig) -> RecorderResult<DatabaseService> {
  tracing::info!("enable testcontainers feature, build testing database service in testcontainers...");

  use testcontainers::{ImageExt, runners::AsyncRunner};
  use testcontainers_ext::{ImageDefaultLogConsumerExt, ImagePruneExistedLabelExt};
  use testcontainers_modules::postgres::Postgres;

  let container = Postgres::default()
    .with_db_name("konobangu")
    .with_user("konobangu")
    .with_password("konobangu")
    .with_tag("18-alpine")
    .with_default_log_consumer()
    .with_prune_existed_label(env!("CARGO_PKG_NAME"), "postgres", true, false)
    .await?;

  let container = container.start().await?;

  let host_ip = container.get_host().await?;
  let host_port = container.get_host_port_ipv4(5432).await?;

  let connection_string = format!("postgres://konobangu:konobangu@{host_ip}:{host_port}/konobangu");

  tracing::debug!("testing database service connection string: {}", connection_string);

  let mut db_service = DatabaseService::from_config(DatabaseConfig {
    uri: connection_string,
    enable_logging: true,
    min_connections: 1,
    max_connections: 5,
    connect_timeout: 5000,
    idle_timeout: 10000,
    acquire_timeout: None,
    auto_migrate: config.auto_migrate,
  })
  .await?;
  db_service.container = Some(container);

  Ok(db_service)
}

#[cfg(not(feature = "testcontainers"))]
pub async fn build_testing_database_service(config: TestingDatabaseServiceConfig) -> RecorderResult<DatabaseService> {
  let db_service = DatabaseService::from_config(DatabaseConfig {
    uri: String::from("postgres://konobangu:konobangu@127.0.0.1:5432/konobangu"),
    enable_logging: true,
    min_connections: 1,
    max_connections: 1,
    connect_timeout: 5000,
    idle_timeout: 10000,
    acquire_timeout: None,
    auto_migrate: config.auto_migrate,
  })
  .await?;

  Ok(db_service)
}
