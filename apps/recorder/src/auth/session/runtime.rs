use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, EntityTrait};
use securitydept_session_context::{ResolvedSessionContextConfig, SessionContextConfig, SessionContextConfigSource};
use sqlx::Row;
use tower_sessions::{Session, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use uuid::Uuid;

use super::store::PersistentSessionStore;
use crate::{
  auth::{AuthError, config::AuthSessionConfig},
  database::roles::{AUTH_ACCESS_ROLE, TASK_CONTROL_ACCESS_ROLE, restrict_pool},
  models::auth::{self, Entity},
};

pub const GRANT_KEY: &str = "konobangu.login_grant";

pub struct SessionRuntime {
  pub config: AuthSessionConfig,
  pub identity_db: DatabaseConnection,
  pub session_pool: sqlx08::PgPool,
  pub sdk_config: ResolvedSessionContextConfig,
  pub store: PersistentSessionStore,
}

impl SessionRuntime {
  pub async fn new(config: AuthSessionConfig) -> Result<Self, AuthError> {
    let invalid = |message: &str| AuthError::SessionConfiguration { message: message.into() };
    let base = &config.external_base_url;
    if !matches!(base.scheme(), "http" | "https")
      || base.host().is_none()
      || base.path() != "/"
      || base.query().is_some()
      || base.fragment().is_some()
      || !base.username().is_empty()
      || base.password().is_some()
    {
      return Err(invalid("external_base_url must be a fixed HTTP(S) origin"));
    }
    if !config.session_cookie_secure && !(base.scheme() == "http" && matches!(base.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))) {
      return Err(invalid("insecure cookies are allowed only for HTTP loopback testing"));
    }
    if config.session_idle_seconds == 0
      || config.session_absolute_seconds <= 0
      || config.session_pending_seconds <= 0
      || config.session_cleanup_seconds == 0
      || config.session_idle_seconds > i64::MAX as u64
    {
      return Err(invalid("session lifetimes and cleanup interval must be positive"));
    }
    let mut options = ConnectOptions::new(&config.identity_database_uri);
    options.sqlx_logging(false).max_connections(3).min_connections(1);
    restrict_pool(&mut options, AUTH_ACCESS_ROLE);
    let identity_db = Database::connect(options).await.map_err(|_| AuthError::SessionStorage)?;
    let initialized = async {
      let pool = identity_db.get_postgres_connection_pool();
      let role = sqlx::query(
        "SELECT rolsuper, rolbypassrls, EXISTS (SELECT 1 FROM pg_class WHERE relname IN ('auth', 'session', 'pending_oauth', 'login_grant') AND \
         pg_has_role(current_user, relowner, 'MEMBER')) AS owns_tables FROM pg_roles WHERE rolname = current_user",
      )
      .fetch_one(pool)
      .await
      .map_err(|_| AuthError::SessionStorage)?;
      if role.get::<bool, _>("rolsuper") || role.get::<bool, _>("rolbypassrls") || role.get::<bool, _>("owns_tables") {
        return Err(invalid("auth pool requires a non-owner NOSUPERUSER NOBYPASSRLS role"));
      }
      let private_access: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'public' AND c.relname IN \
         ('subscriptions','bangumi','episodes','subscription_bangumi','subscription_episode','downloaders','downloads','credential3rd','feeds','cron','\
         subscriber_tasks','system_tasks') AND has_table_privilege(current_user, c.oid, 'SELECT,INSERT,UPDATE,DELETE')) OR has_schema_privilege(current_user, \
         'apalis', 'USAGE') OR has_schema_privilege(current_user, 'apalis_legacy_v07', 'USAGE') OR pg_has_role(current_user, $1::text, 'MEMBER')",
      )
      .bind(TASK_CONTROL_ACCESS_ROLE)
      .fetch_one(pool)
      .await
      .map_err(|_| AuthError::SessionStorage)?;
      if private_access {
        return Err(invalid("auth pool must not have private application or task-control privileges"));
      }
      let sdk_config = SessionContextConfig::builder()
        .cookie_name("konobangu_session".into())
        .secure(config.session_cookie_secure)
        .http_only(true)
        .cookie_path("/".into())
        .ttl(Some(Duration::from_secs(config.session_idle_seconds)))
        .build()
        .resolve_all()
        .map_err(|_| invalid("session cookie configuration is invalid"))?;
      let session_pool = sqlx08::postgres::PgPoolOptions::new()
        .max_connections(2)
        .min_connections(1)
        .after_connect(|connection, _| {
          Box::pin(async move {
            sqlx08::query("SELECT set_config('role', $1, false)")
              .bind(AUTH_ACCESS_ROLE)
              .execute(connection)
              .await?;
            Ok(())
          })
        })
        .before_acquire(|connection, _| {
          Box::pin(async move {
            sqlx08::query("SELECT set_config('role', $1, false)")
              .bind(AUTH_ACCESS_ROLE)
              .execute(connection)
              .await?;
            Ok(true)
          })
        })
        .connect(&config.identity_database_uri)
        .await
        .map_err(|_| AuthError::SessionStorage)?;
      let store = match PostgresStore::new(session_pool.clone()).with_schema_name("auth_session") {
        Ok(store) => PersistentSessionStore(store),
        Err(_) => {
          session_pool.close().await;
          return Err(invalid("session schema is invalid"));
        }
      };
      Ok((sdk_config, session_pool, store))
    }
    .await;
    let (sdk_config, session_pool, store) = match initialized {
      Ok(value) => value,
      Err(error) => {
        let _ = identity_db.close().await;
        return Err(error);
      }
    };
    Ok(Self {
      config,
      identity_db,
      session_pool,
      sdk_config,
      store,
    })
  }

  pub async fn close(&self) -> Result<(), AuthError> {
    self.session_pool.close().await;
    self.identity_db.clone().close().await.map_err(|_| AuthError::SessionStorage)
  }

  pub fn layer(&self) -> SessionManagerLayer<PersistentSessionStore> {
    securitydept_session_context::build_session_layer(&self.sdk_config, self.store.clone()).with_always_save(true)
  }

  pub async fn authenticate(&self, session: &Session) -> Result<auth::Model, AuthError> {
    let id: Uuid = session
      .get(GRANT_KEY)
      .await
      .map_err(|_| AuthError::SessionStorage)?
      .ok_or(AuthError::SessionUnauthorized)?;
    let auth_id: Option<i32> =
      sqlx::query_scalar("SELECT auth_id FROM auth_session.login_grant WHERE id = $1 AND revoked_at IS NULL AND expires_at > CURRENT_TIMESTAMP")
        .bind(id)
        .fetch_optional(self.identity_db.get_postgres_connection_pool())
        .await
        .map_err(|_| AuthError::SessionStorage)?;
    Entity::find_by_id(auth_id.ok_or(AuthError::SessionUnauthorized)?)
      .one(&self.identity_db)
      .await
      .map_err(|_| AuthError::SessionStorage)?
      .ok_or(AuthError::SessionUnauthorized)
  }

  pub async fn create_grant(&self, auth_id: i32) -> Result<Uuid, AuthError> {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO auth_session.login_grant (id, auth_id, expires_at) VALUES ($1, $2, CURRENT_TIMESTAMP + $3 * INTERVAL '1 second')")
      .bind(id)
      .bind(auth_id)
      .bind(self.config.session_absolute_seconds)
      .execute(self.identity_db.get_postgres_connection_pool())
      .await
      .map_err(|_| AuthError::SessionStorage)?;
    Ok(id)
  }

  pub async fn revoke(&self, session: &Session) -> Result<(), AuthError> {
    if let Some(id) = session.get::<Uuid>(GRANT_KEY).await.map_err(|_| AuthError::SessionStorage)? {
      sqlx::query("UPDATE auth_session.login_grant SET revoked_at = COALESCE(revoked_at, CURRENT_TIMESTAMP) WHERE id = $1")
        .bind(id)
        .execute(self.identity_db.get_postgres_connection_pool())
        .await
        .map_err(|_| AuthError::SessionStorage)?;
    }
    session.flush().await.map_err(|_| AuthError::SessionStorage)
  }

  pub async fn cleanup_loop(&self) {
    let mut interval = tokio::time::interval(Duration::from_secs(self.config.session_cleanup_seconds));
    loop {
      interval.tick().await;
      if self.cleanup().await.is_err() {
        tracing::error!("Authentication storage cleanup failed");
      }
    }
  }

  pub async fn cleanup(&self) -> Result<(), AuthError> {
    use tower_sessions_core_014::session_store::ExpiredDeletion;
    self.store.0.delete_expired().await.map_err(|_| AuthError::SessionStorage)?;
    let pool = self.identity_db.get_postgres_connection_pool();
    sqlx::query("DELETE FROM auth_session.pending_oauth WHERE expires_at <= CURRENT_TIMESTAMP")
      .execute(pool)
      .await
      .map_err(|_| AuthError::SessionStorage)?;
    // Revoked grants remain until their absolute deadline, blocking late
    // session saves.
    sqlx::query("DELETE FROM auth_session.login_grant WHERE expires_at <= CURRENT_TIMESTAMP")
      .execute(pool)
      .await
      .map_err(|_| AuthError::SessionStorage)?;
    Ok(())
  }
}
