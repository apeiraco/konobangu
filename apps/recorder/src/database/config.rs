use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
#[serde(from = "DatabaseWire", into = "DatabaseWire")]
pub struct DatabaseConfig {
  pub uri: String,
  pub enable_logging: bool,
  pub min_connections: u32,
  pub max_connections: u32,
  pub connect_timeout: u64,
  pub idle_timeout: u64,
  pub acquire_timeout: Option<u64>,
  #[serde(default)]
  pub auto_migrate: bool,
  #[serde(default)]
  pub legacy_oidc_issuer: Option<String>,
  #[serde(default)]
  pub migration_database_uri: Option<String>,
}

impl std::fmt::Debug for DatabaseConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut debug = f.debug_struct("DatabaseConfig");
    debug.field("enable_logging", &self.enable_logging);
    debug.field("min_connections", &self.min_connections);
    debug.field("max_connections", &self.max_connections);
    debug.field("connect_timeout", &self.connect_timeout);
    debug.field("idle_timeout", &self.idle_timeout);
    debug.field("acquire_timeout", &self.acquire_timeout);
    debug.field("auto_migrate", &self.auto_migrate);
    debug.field("legacy_oidc_issuer", &self.legacy_oidc_issuer);
    debug.field("uri", &"[REDACTED]");
    debug.field("migration_database_uri", &"[REDACTED]");
    debug.finish()
  }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DatabaseWire {
  url: String,
  #[serde(default)]
  pool: PoolWire,
  #[serde(default)]
  migration: MigrationWire,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct PoolWire {
  log_queries: bool,
  min_connections: u32,
  max_connections: u32,
  connect_timeout_ms: u64,
  idle_timeout_ms: u64,
  acquire_timeout_ms: Option<u64>,
}
impl Default for PoolWire {
  fn default() -> Self {
    Self {
      log_queries: false,
      min_connections: 1,
      max_connections: 5,
      connect_timeout_ms: 5000,
      idle_timeout_ms: 60000,
      acquire_timeout_ms: None,
    }
  }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct MigrationWire {
  url: Option<String>,
  auto_run: bool,
  legacy_oidc_issuer: Option<String>,
}
impl Default for MigrationWire {
  fn default() -> Self {
    Self {
      url: None,
      auto_run: true,
      legacy_oidc_issuer: None,
    }
  }
}
impl From<DatabaseWire> for DatabaseConfig {
  fn from(w: DatabaseWire) -> Self {
    Self {
      uri: w.url,
      enable_logging: w.pool.log_queries,
      min_connections: w.pool.min_connections,
      max_connections: w.pool.max_connections,
      connect_timeout: w.pool.connect_timeout_ms,
      idle_timeout: w.pool.idle_timeout_ms,
      acquire_timeout: w.pool.acquire_timeout_ms,
      auto_migrate: w.migration.auto_run,
      legacy_oidc_issuer: w.migration.legacy_oidc_issuer,
      migration_database_uri: w.migration.url,
    }
  }
}
impl From<DatabaseConfig> for DatabaseWire {
  fn from(c: DatabaseConfig) -> Self {
    Self {
      url: c.uri,
      pool: PoolWire {
        log_queries: c.enable_logging,
        min_connections: c.min_connections,
        max_connections: c.max_connections,
        connect_timeout_ms: c.connect_timeout,
        idle_timeout_ms: c.idle_timeout,
        acquire_timeout_ms: c.acquire_timeout,
      },
      migration: MigrationWire {
        url: c.migration_database_uri,
        auto_run: c.auto_migrate,
        legacy_oidc_issuer: c.legacy_oidc_issuer,
      },
    }
  }
}
