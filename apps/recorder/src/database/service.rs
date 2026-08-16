use std::{ops::Deref, time::Duration};

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, ExecResult, QueryResult, Statement, TransactionTrait};
use sea_orm_migration::MigratorTrait;

use super::{DatabaseConfig, roles::TASK_CONTROL_ACCESS_ROLE};
use crate::{errors::RecorderResult, migrations::Migrator};

pub struct DatabaseService {
  pub config: DatabaseConfig,
  connection: DatabaseConnection,
  #[cfg(feature = "testcontainers")]
  pub container: Option<testcontainers::ContainerAsync<testcontainers_modules::postgres::Postgres>>,
}

impl DatabaseService {
  pub async fn from_app_config(config: &crate::app::AppConfig) -> RecorderResult<Self> {
    let roles = config
      .database
      .auto_migrate
      .then(|| crate::migrations::access::RuntimeLogins::from_config(config))
      .transpose()?;
    Self::runtime(config.database.clone(), roles).await
  }

  pub async fn from_config(config: DatabaseConfig) -> RecorderResult<Self> {
    Self::runtime(config, None).await
  }

  async fn runtime(mut config: DatabaseConfig, roles: Option<crate::migrations::access::RuntimeLogins>) -> RecorderResult<Self> {
    if config.auto_migrate {
      let mut migration = config.clone();
      migration.uri = config.migration_database_uri.clone().unwrap_or_else(|| config.uri.clone());
      let owner = Self::from_migration_config(migration).await?;
      let result = async {
        if let Some(roles) = &roles {
          crate::migrations::access::grant_runtime_memberships(owner.as_ref(), roles).await
        } else {
          crate::migrations::access::grant_membership(
            owner.as_ref(),
            super::roles::APP_SCOPED_ACCESS_ROLE,
            &crate::migrations::access::login(&config.uri)?,
          )
          .await
        }
      }
      .await;
      owner.as_ref().clone().close().await?;
      result?;
      config.auto_migrate = false;
    }
    let me = Self::connect(config, Some(super::roles::APP_SCOPED_ACCESS_ROLE)).await?;
    let row = me
      .query_one_raw(Statement::from_string(
        DbBackend::Postgres,
        "SELECT rolsuper, rolbypassrls, EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname IN ('public', \
         'auth_identity', 'auth_session', 'apalis') AND pg_has_role(current_user, c.relowner, 'MEMBER')) AS owns_tables FROM pg_roles WHERE rolname = \
         current_user",
      ))
      .await?
      .ok_or_else(|| DbErr::Custom("Application database role was not found".into()))?;
    if row.try_get::<bool>("", "rolsuper")? || row.try_get::<bool>("", "rolbypassrls")? || row.try_get::<bool>("", "owns_tables")? {
      return Err(DbErr::Custom("App-scoped pool requires a non-owner NOSUPERUSER NOBYPASSRLS role".into()).into());
    }
    let identity_access = me
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT has_schema_privilege(current_user, 'auth_identity', 'USAGE') OR has_schema_privilege(current_user, 'auth_session', 'USAGE') OR \
         pg_has_role(current_user, $1::text, 'MEMBER') OR has_schema_privilege(current_user, 'apalis', 'USAGE') OR has_schema_privilege(current_user, \
         'apalis_legacy_v07', 'USAGE') AS identity_access",
        [TASK_CONTROL_ACCESS_ROLE.into()],
      ))
      .await?
      .ok_or_else(|| DbErr::Custom("Identity schema permission check failed".into()))?;
    if identity_access.try_get::<bool>("", "identity_access")? {
      return Err(DbErr::Custom("App-scoped pool must not have auth or task-control access".into()).into());
    }
    Ok(me)
  }

  /// Explicit owner-only migration/fixture connection; never inserted into
  /// AppContext.
  pub async fn from_migration_config(config: DatabaseConfig) -> RecorderResult<Self> {
    Self::connect(config, None).await
  }

  async fn connect(config: DatabaseConfig, role: Option<&'static str>) -> RecorderResult<Self> {
    let db_config = config.clone();
    let mut opt = ConnectOptions::new(&config.uri);
    opt
      .max_connections(config.max_connections)
      .min_connections(config.min_connections)
      .connect_timeout(Duration::from_millis(config.connect_timeout))
      .idle_timeout(Duration::from_millis(config.idle_timeout))
      .sqlx_logging(config.enable_logging);

    if let Some(acquire_timeout) = config.acquire_timeout {
      opt.acquire_timeout(Duration::from_millis(acquire_timeout));
    }

    if let Some(role) = role {
      super::roles::restrict_pool(&mut opt, role);
    }
    let db = Database::connect(opt)
      .await
      .map_err(|_| DbErr::Custom("Database connection failed (details redacted)".into()))?;

    let me = Self {
      connection: db,
      #[cfg(feature = "testcontainers")]
      container: None,
      config: db_config,
    };

    if config.auto_migrate {
      me.migrate_up().await?;
    }

    Ok(me)
  }

  pub async fn migrate_up(&self) -> RecorderResult<()> {
    let report = crate::migrations::legacy::preflight(&self.connection).await?;
    if !report.issues.is_empty() {
      return Err(DbErr::Migration(serde_json::to_string(&report).map_err(|e| DbErr::Custom(e.to_string()))?).into());
    }
    crate::migrations::legacy::bootstrap(&self.connection).await?;
    let transaction = self.connection.begin().await?;
    transaction
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT set_config('konobangu.legacy_oidc_issuer', $1, true)",
        [self.config.legacy_oidc_issuer.clone().unwrap_or_default().into()],
      ))
      .await?;
    Migrator::up(&transaction, None).await?;
    transaction.commit().await?;
    crate::migrations::legacy::initialize_queue(&self.connection).await?;
    // The mature store owns its SQL and encoding; DDL runs only in this owner
    // phase.
    let session_pool = sqlx08::postgres::PgPoolOptions::new()
      .max_connections(1)
      .connect(&self.config.uri)
      .await
      .map_err(|_| DbErr::Custom("Migration session connection failed (details redacted)".into()))?;
    let migrated = async {
      tower_sessions_sqlx_store::PostgresStore::new(session_pool.clone())
        .with_schema_name("auth_session")
        .map_err(DbErr::Custom)?
        .migrate()
        .await
        .map_err(|error| DbErr::Custom(error.to_string()))
    }
    .await;
    session_pool.close().await;
    migrated?;
    crate::migrations::access::initialize_access(&self.connection).await?;
    Ok(())
  }

  pub async fn migrate_down(&self) -> RecorderResult<()> {
    Migrator::down(&self.connection, None).await?;
    {
      self.execute_unprepared(r#"DROP SCHEMA IF EXISTS apalis CASCADE"#).await?;
    }
    Ok(())
  }
}

impl Deref for DatabaseService {
  type Target = DatabaseConnection;

  fn deref(&self) -> &Self::Target {
    &self.connection
  }
}

impl AsRef<DatabaseConnection> for DatabaseService {
  fn as_ref(&self) -> &DatabaseConnection {
    &self.connection
  }
}

#[async_trait::async_trait]
impl ConnectionTrait for DatabaseService {
  fn get_database_backend(&self) -> DbBackend {
    self.deref().get_database_backend()
  }

  async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
    self.deref().execute_raw(stmt).await
  }

  async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
    self.deref().execute_unprepared(sql).await
  }

  async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
    self.deref().query_one_raw(stmt).await
  }

  async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
    self.deref().query_all_raw(stmt).await
  }
}
