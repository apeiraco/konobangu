use sea_orm::{ConnectOptions, ConnectionTrait, DatabaseTransaction, DbErr};

pub const APP_SCOPED_ACCESS_ROLE: &str = "konobangu_app_scoped_access";
pub const AUTH_ACCESS_ROLE: &str = "konobangu_auth_access";
pub const TASK_CONTROL_ACCESS_ROLE: &str = "konobangu_task_control_access";
pub const CAPABILITY_ROLES: [&str; 3] = [APP_SCOPED_ACCESS_ROLE, AUTH_ACCESS_ROLE, TASK_CONTROL_ACCESS_ROLE];

/// Library queries acquire their own connections. Every physical connection
/// therefore has a restricted baseline, restored before each pool checkout.
pub fn restrict_pool(options: &mut ConnectOptions, role: &'static str) {
  assert!(CAPABILITY_ROLES.contains(&role));
  options.map_sqlx_postgres_pool_opts(move |pool| {
    pool
      .after_connect(move |connection, _| {
        Box::pin(async move {
          sqlx::query("SELECT set_config('role', $1, false)").bind(role).execute(connection).await?;
          Ok(())
        })
      })
      .before_acquire(move |connection, _| {
        Box::pin(async move {
          sqlx::query("SELECT set_config('role', $1, false)").bind(role).execute(connection).await?;
          Ok(true)
        })
      })
  });
}

/// Role and subscriber scope are transaction-local; completion or cancellation
/// returns to the pool's restricted baseline.
pub async fn set_local_role(db: &DatabaseTransaction, role: &'static str) -> Result<(), DbErr> {
  assert!(CAPABILITY_ROLES.contains(&role));
  db.execute_unprepared(&format!("SET LOCAL ROLE {role}")).await?;
  Ok(())
}
