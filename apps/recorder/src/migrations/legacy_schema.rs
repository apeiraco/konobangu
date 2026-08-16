//! Minimal empty compatibility tables for replaying this project's old
//! migrations. Existing Apalis tables are never rebuilt; its runtime/functions
//! are unnecessary.
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .get_connection()
      .execute_unprepared(
        r#"
      CREATE SCHEMA apalis;
      CREATE TABLE apalis.workers (
        id text PRIMARY KEY, worker_type text NOT NULL, storage_name text NOT NULL,
        layers text NOT NULL DEFAULT '', last_seen timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
      );
      CREATE TABLE apalis.jobs (
        id text PRIMARY KEY, job jsonb NOT NULL, job_type text NOT NULL,
        status text NOT NULL DEFAULT 'Pending', attempts integer NOT NULL DEFAULT 0,
        max_attempts integer NOT NULL DEFAULT 25, priority integer NOT NULL DEFAULT 0,
        run_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP, done_at timestamptz,
        last_error text, lock_at timestamptz, lock_by text REFERENCES apalis.workers(id)
      );
    "#,
      )
      .await?;
    Ok(())
  }
  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    Err(DbErr::Migration("Restore the pre-upgrade backup instead of dropping legacy task data".into()))
  }
}
