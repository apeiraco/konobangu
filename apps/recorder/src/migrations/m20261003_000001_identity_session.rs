use sea_orm_migration::prelude::*;

use super::defs::ApplicationSchema;
use crate::database::roles::TASK_CONTROL_ACCESS_ROLE;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    super::access::create_capability_roles(manager.get_connection()).await?;
    // The owner must explicitly attribute every legacy OIDC mapping before DDL.
    manager
      .get_connection()
      .execute_unprepared(&format!(
        r#"
DO $$ BEGIN
    IF EXISTS (SELECT 1 FROM public.auth GROUP BY auth_type, pid HAVING count(*) > 1) THEN
        RAISE EXCEPTION 'Conflicting legacy identity keys; resolve attribution before migration';
    END IF;
    IF EXISTS (SELECT 1 FROM public.auth WHERE auth_type = 'oidc')
       AND NULLIF(current_setting('konobangu.legacy_oidc_issuer', true), '') IS NULL THEN
        RAISE EXCEPTION 'Legacy OIDC identities require database.legacy_oidc_issuer; migration has not changed data';
    END IF;
END $$;
CREATE SCHEMA {auth_identity};
CREATE SCHEMA {auth_session};
ALTER TABLE public.auth ADD COLUMN issuer text;
UPDATE public.auth SET issuer = current_setting('konobangu.legacy_oidc_issuer', true) WHERE auth_type = 'oidc';
ALTER TABLE public.auth ADD CONSTRAINT auth_issuer_type CHECK (
    (auth_type = 'basic' AND issuer IS NULL) OR (auth_type = 'oidc' AND issuer IS NOT NULL AND issuer <> '')
);
DROP INDEX public.idx_auth_pid_auth_type;
CREATE UNIQUE INDEX {auth_identity}_key ON public.auth (auth_type, issuer, pid) NULLS NOT DISTINCT;
ALTER TABLE public.auth DISABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth NO FORCE ROW LEVEL SECURITY;
ALTER TABLE public.auth SET SCHEMA {auth_identity};
REVOKE ALL ON SCHEMA {auth_identity}, {auth_session} FROM PUBLIC;
CREATE TABLE {auth_session}.pending_oauth (
    state text PRIMARY KEY, nonce text NOT NULL, code_verifier text,
    extra_data jsonb, expires_at timestamptz NOT NULL
);
CREATE INDEX pending_oauth_expiry ON {auth_session}.pending_oauth (expires_at);
CREATE TABLE {auth_session}.login_grant (
    id uuid PRIMARY KEY, auth_id integer NOT NULL REFERENCES {auth_identity}.auth(id),
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at timestamptz NOT NULL, revoked_at timestamptz,
    CHECK (expires_at > created_at)
);
CREATE INDEX login_grant_expiry ON {auth_session}.login_grant (expires_at);
"#,
        auth_identity = ApplicationSchema::AuthIdentity.to_string(),
        auth_session = ApplicationSchema::AuthSession.to_string(),
      ))
      .await?;
    let setting = "NULLIF(current_setting('app.subscriber_id', true), '')::integer";
    for table in [
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
    ] {
      let select = if matches!(table, "feeds" | "cron") {
        format!("subscriber_id IS NULL OR subscriber_id = {setting}")
      } else {
        format!("subscriber_id = {setting}")
      };
      manager
        .get_connection()
        .execute_unprepared(&format!(
          "ALTER POLICY {table}_select ON {table} USING ({select}); ALTER POLICY {table}_insert ON {table} WITH CHECK (subscriber_id = {setting}); ALTER \
           POLICY {table}_update ON {table} USING (subscriber_id = {setting}) WITH CHECK (subscriber_id = {setting}); ALTER POLICY {table}_delete ON {table} \
           USING (subscriber_id = {setting});"
        ))
        .await?;
    }
    manager
      .get_connection()
      .execute_unprepared(&format!(
        "CREATE POLICY cron_task_control ON cron TO {TASK_CONTROL_ACCESS_ROLE} USING (true) WITH CHECK (true)"
      ))
      .await?;
    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    Err(DbErr::Migration(
      "Identity keys cannot safely be collapsed across issuers; restore the pre-migration backup".into(),
    ))
  }
}
