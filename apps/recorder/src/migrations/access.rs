//! Owner-side runtime grants. Application object names come from migration
//! identifiers.
use sea_orm::{
  ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement, TransactionTrait,
  sea_query::{Alias, Iden, IntoIden, PostgresQueryBuilder, QuotedBuilder},
};

use super::defs::{
  ApalisSchema, ApplicationSchema, Auth, Bangumi, Credential3rd, Cron, Downloaders, Downloads, Episodes, Feeds, Subscribers, SubscriptionBangumi,
  SubscriptionEpisode, Subscriptions, TaskOutbox, TaskRuns, TaskViews,
};
use crate::{
  app::AppConfig,
  database::roles::{APP_SCOPED_ACCESS_ROLE, AUTH_ACCESS_ROLE, CAPABILITY_ROLES, TASK_CONTROL_ACCESS_ROLE},
};

pub struct RuntimeLogins {
  pub app_scoped: String,
  pub auth: String,
  pub task_control: Option<String>,
}

pub(crate) fn login(uri: &str) -> Result<String, DbErr> {
  let invalid = || DbErr::Custom("Role provisioning requires PostgreSQL URLs with explicit UTF-8 login names".into());
  let url = url::Url::parse(uri).map_err(|_| invalid())?;
  if !matches!(url.scheme(), "postgres" | "postgresql") {
    return Err(invalid());
  }
  let name = percent_encoding::percent_decode_str(url.username()).decode_utf8().map_err(|_| invalid())?;
  if name.is_empty() || name.len() > 63 || name.contains('\0') {
    return Err(invalid());
  }
  Ok(name.into_owned())
}

impl RuntimeLogins {
  pub fn from_config(config: &AppConfig) -> Result<Self, DbErr> {
    Ok(Self {
      app_scoped: login(&config.database.uri)?,
      auth: login(&config.auth.session().identity_database_uri)?,
      task_control: config.task.queue_database_uri.as_deref().map(login).transpose()?,
    })
  }
}

fn identifier(name: impl IntoIden) -> String {
  let mut sql = String::new();
  PostgresQueryBuilder.prepare_iden(&name.into_iden(), &mut sql);
  sql
}

fn public_table(table: impl IntoIden) -> String {
  format!("{}.{}", identifier(ApplicationSchema::Public), identifier(table))
}

/// Third-party schemas must be initialized before their runtime grants are
/// applied.
pub async fn grant_task_control_access(db: &impl ConnectionTrait) -> Result<(), DbErr> {
  let role = identifier(Alias::new(TASK_CONTROL_ACCESS_ROLE));
  let public = identifier(ApplicationSchema::Public);
  let queue = identifier(ApalisSchema::Schema);
  db.execute_unprepared(&format!(
    "GRANT USAGE ON SCHEMA {public}, {queue} TO {role};
     GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA {queue} TO {role};
     GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA {queue} TO {role};
     GRANT SELECT, INSERT, UPDATE, DELETE ON {cron} TO {role};
     GRANT SELECT, INSERT, UPDATE ON {runs}, {outbox} TO {role};",
    cron = public_table(Cron::Table),
    runs = public_table(TaskRuns::Table),
    outbox = public_table(TaskOutbox::Table),
  ))
  .await?;
  Ok(())
}

/// Capability roles are cluster objects; migrations validate rather than
/// silently repurpose an existing privileged role.
pub async fn create_capability_roles(db: &impl ConnectionTrait) -> Result<(), DbErr> {
  for name in CAPABILITY_ROLES {
    let role = identifier(Alias::new(name));
    db.execute_unprepared(&format!(
      "DO $$ BEGIN
         IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = '{name}') THEN
           CREATE ROLE {role} NOLOGIN NOINHERIT NOSUPERUSER NOBYPASSRLS;
         END IF;
         IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = '{name}' AND (rolcanlogin OR rolsuper OR rolbypassrls)) THEN
           RAISE EXCEPTION 'Capability role must be NOLOGIN NOSUPERUSER NOBYPASSRLS';
         END IF;
       END $$;"
    ))
    .await?;
  }
  Ok(())
}

/// Only explicit SET ROLE activates capabilities; ordinary membership never
/// combines app, auth and task-control privileges.
pub async fn grant_runtime_memberships(db: &DatabaseConnection, roles: &RuntimeLogins) -> Result<(), DbErr> {
  let txn = db.begin().await?;
  for (capability, login) in [
    (APP_SCOPED_ACCESS_ROLE, Some(roles.app_scoped.as_str())),
    (AUTH_ACCESS_ROLE, Some(roles.auth.as_str())),
    (TASK_CONTROL_ACCESS_ROLE, roles.task_control.as_deref()),
  ] {
    let Some(login) = login else { continue };
    grant_membership(&txn, capability, login).await?;
  }
  txn.commit().await
}

pub(crate) async fn grant_membership(db: &impl ConnectionTrait, capability: &'static str, login: &str) -> Result<(), DbErr> {
  assert!(CAPABILITY_ROLES.contains(&capability));
  if CAPABILITY_ROLES.contains(&login) {
    return Err(DbErr::Custom("Connection login must be distinct from NOLOGIN capability roles".into()));
  }
  db.execute_unprepared(&format!(
    "GRANT {} TO {} WITH INHERIT FALSE, SET TRUE",
    identifier(Alias::new(capability)),
    identifier(Alias::new(login)),
  ))
  .await?;
  Ok(())
}

/// Initialize grants only after application and library DDL has completed.
pub async fn initialize_access(db: &DatabaseConnection) -> Result<(), DbErr> {
  create_capability_roles(db).await?;
  let txn = db.begin().await?;
  let app = identifier(Alias::new(APP_SCOPED_ACCESS_ROLE));
  let auth = identifier(Alias::new(AUTH_ACCESS_ROLE));
  let public = identifier(ApplicationSchema::Public);
  let identity = identifier(ApplicationSchema::AuthIdentity);
  let session = identifier(ApplicationSchema::AuthSession);
  let writable = [
    Subscriptions::Table.into_iden(),
    Bangumi::Table.into_iden(),
    Episodes::Table.into_iden(),
    SubscriptionBangumi::Table.into_iden(),
    SubscriptionEpisode::Table.into_iden(),
    Downloaders::Table.into_iden(),
    Downloads::Table.into_iden(),
    Credential3rd::Table.into_iden(),
    Feeds::Table.into_iden(),
    Cron::Table.into_iden(),
  ]
  .into_iter()
  .map(public_table)
  .collect::<Vec<_>>()
  .join(", ");
  txn
    .execute_unprepared(&format!(
      "GRANT USAGE ON SCHEMA {public} TO {app};
     GRANT SELECT, INSERT, UPDATE, DELETE ON {writable} TO {app};
     GRANT SELECT ON {subscribers}, {subscriber_tasks}, {system_tasks} TO {app};
     GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA {public} TO {app};
     GRANT SELECT, INSERT, UPDATE ON {runs}, {outbox} TO {app};
     GRANT USAGE ON SCHEMA {public}, {identity}, {session} TO {auth};
     GRANT SELECT, INSERT ON {identity}.{auth_table} TO {auth};
     GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA {identity} TO {auth};
     GRANT INSERT ({display_name}), SELECT ({id}) ON {subscribers} TO {auth};
     GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA {session} TO {auth};",
      subscribers = public_table(Subscribers::Table),
      subscriber_tasks = public_table(TaskViews::SubscriberTasks),
      system_tasks = public_table(TaskViews::SystemTasks),
      runs = public_table(TaskRuns::Table),
      outbox = public_table(TaskOutbox::Table),
      auth_table = identifier(Auth::Table),
      display_name = identifier(Subscribers::DisplayName),
      id = identifier(Subscribers::Id),
    ))
    .await?;
  let sequence: Option<String> = txn
    .query_one_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "SELECT pg_get_serial_sequence($1, $2) AS name",
      [public_table(Subscribers::Table).into(), Subscribers::Id.to_string().into()],
    ))
    .await?
    .ok_or_else(|| DbErr::Custom("Subscriber sequence lookup failed".into()))?
    .try_get("", "name")?;
  let sequence = sequence.ok_or_else(|| DbErr::Custom("Subscriber ID requires its migration-owned sequence".into()))?;
  txn.execute_unprepared(&format!("GRANT USAGE, SELECT ON SEQUENCE {sequence} TO {auth}")).await?;
  grant_task_control_access(&txn).await?;
  for role in CAPABILITY_ROLES {
    txn
      .execute_unprepared(&format!("GRANT {} TO CURRENT_USER WITH INHERIT FALSE, SET TRUE", identifier(Alias::new(role)),))
      .await?;
  }
  txn.commit().await
}
