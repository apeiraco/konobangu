use async_trait::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use super::defs::{ApalisJobs, ApalisSchema};
use crate::{
  migrations::defs::{Subscribers, Subscriptions},
  task::{
    SETUP_APALIS_JOBS_EXTRA_FOREIGN_KEYS_FUNCTION_NAME, SETUP_APALIS_JOBS_EXTRA_FOREIGN_KEYS_TRIGGER_NAME, SUBSCRIBER_TASK_APALIS_NAME, SYSTEM_TASK_APALIS_NAME,
  },
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        TableAlterStatement::new()
          .table((ApalisSchema::Schema, ApalisJobs::Table))
          .add_column_if_not_exists(integer_null(ApalisJobs::SubscriberId))
          .add_column_if_not_exists(integer_null(ApalisJobs::SubscriptionId))
          .add_column_if_not_exists(text_null(ApalisJobs::TaskType))
          .add_foreign_key(
            TableForeignKey::new()
              .name("fk_apalis_jobs_subscriber_id")
              .from_tbl((ApalisSchema::Schema, ApalisJobs::Table))
              .from_col(ApalisJobs::SubscriberId)
              .to_tbl(Subscribers::Table)
              .to_col(Subscribers::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Restrict),
          )
          .add_foreign_key(
            TableForeignKey::new()
              .name("fk_apalis_jobs_subscription_id")
              .from_tbl((ApalisSchema::Schema, ApalisJobs::Table))
              .from_col(ApalisJobs::SubscriptionId)
              .to_tbl(Subscriptions::Table)
              .to_col(Subscriptions::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Restrict),
          )
          .to_owned(),
      )
      .await?;

    let db = manager.get_connection();

    db.execute_unprepared(&format!(
            r#"UPDATE {apalis_schema}.{apalis_table} SET {subscriber_id} = ({job} ->> '{subscriber_id}')::integer, {task_type} = ({job} ->> '{task_type}')::text, {subscription_id} = ({job} ->> '{subscription_id}')::integer"#,
            apalis_schema = ApalisSchema::Schema.to_string(),
            apalis_table = ApalisJobs::Table.to_string(),
            subscriber_id = ApalisJobs::SubscriberId.to_string(),
            job = ApalisJobs::Job.to_string(),
            task_type = ApalisJobs::TaskType.to_string(),
            subscription_id = ApalisJobs::SubscriptionId.to_string(),
        )).await?;

    db.execute_unprepared(&format!(
            r#"CREATE OR REPLACE FUNCTION {apalis_schema}.{SETUP_APALIS_JOBS_EXTRA_FOREIGN_KEYS_FUNCTION_NAME}() RETURNS trigger AS $$
            DECLARE
                new_job_subscriber_id integer;
                new_job_subscription_id integer;
                new_job_task_type text;
            BEGIN
                new_job_subscriber_id = (NEW.{job} ->> '{subscriber_id}')::integer;
                new_job_subscription_id = (NEW.{job} ->> '{subscription_id}')::integer;
                new_job_task_type = (NEW.{job} ->> '{task_type}')::text;
                IF new_job_subscriber_id IS DISTINCT FROM (OLD.{job} ->> '{subscriber_id}')::integer AND new_job_subscriber_id IS DISTINCT FROM NEW.{subscriber_id} THEN
                    NEW.{subscriber_id} = new_job_subscriber_id;
                END IF;
                IF new_job_subscription_id IS DISTINCT FROM (OLD.{job} ->> '{subscription_id}')::integer AND new_job_subscription_id IS DISTINCT FROM NEW.{subscription_id} THEN
                    NEW.{subscription_id} = new_job_subscription_id;
                END IF;
                IF new_job_task_type IS DISTINCT FROM (OLD.{job} ->> '{task_type}')::text AND new_job_task_type IS DISTINCT FROM NEW.{task_type} THEN
                    NEW.{task_type} = new_job_task_type;
                END IF;
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;"#,
            apalis_schema = ApalisSchema::Schema.to_string(),
            job = ApalisJobs::Job.to_string(),
            subscriber_id = ApalisJobs::SubscriberId.to_string(),
            subscription_id = ApalisJobs::SubscriptionId.to_string(),
            task_type = ApalisJobs::TaskType.to_string(),
        )).await?;

    db.execute_unprepared(&format!(
      r#"CREATE OR REPLACE TRIGGER {SETUP_APALIS_JOBS_EXTRA_FOREIGN_KEYS_TRIGGER_NAME}
            BEFORE INSERT OR UPDATE ON {apalis_schema}.{apalis_table}
            FOR EACH ROW
            EXECUTE FUNCTION {apalis_schema}.{SETUP_APALIS_JOBS_EXTRA_FOREIGN_KEYS_FUNCTION_NAME}();"#,
      apalis_schema = ApalisSchema::Schema.to_string(),
      apalis_table = ApalisJobs::Table.to_string()
    ))
    .await?;

    db.execute_unprepared(&format!(
      r#"CREATE OR REPLACE VIEW subscriber_tasks AS
                SELECT
                    {job},
                    {job_type},
                    {status},
                    {subscriber_id},
                    {task_type},
                    {id},
                    {attempts},
                    {max_attempts},
                    {run_at},
                    {last_error},
                    {lock_at},
                    {lock_by},
                    {done_at},
                    {priority},
                    {subscription_id}
                FROM {apalis_schema}.{apalis_table}
                WHERE {job_type} = '{SUBSCRIBER_TASK_APALIS_NAME}'
                AND jsonb_path_exists({job}, '$.{subscriber_id} ? (@.type() == "number")')
                AND jsonb_path_exists({job}, '$.{task_type} ? (@.type() == "string")')"#,
      apalis_schema = ApalisSchema::Schema.to_string(),
      apalis_table = ApalisJobs::Table.to_string(),
      job = ApalisJobs::Job.to_string(),
      job_type = ApalisJobs::JobType.to_string(),
      status = ApalisJobs::Status.to_string(),
      subscriber_id = ApalisJobs::SubscriberId.to_string(),
      task_type = ApalisJobs::TaskType.to_string(),
      id = ApalisJobs::Id.to_string(),
      attempts = ApalisJobs::Attempts.to_string(),
      max_attempts = ApalisJobs::MaxAttempts.to_string(),
      run_at = ApalisJobs::RunAt.to_string(),
      last_error = ApalisJobs::LastError.to_string(),
      lock_at = ApalisJobs::LockAt.to_string(),
      lock_by = ApalisJobs::LockBy.to_string(),
      done_at = ApalisJobs::DoneAt.to_string(),
      priority = ApalisJobs::Priority.to_string(),
      subscription_id = ApalisJobs::SubscriptionId.to_string(),
    ))
    .await?;

    db.execute_unprepared(&format!(
      r#"CREATE OR REPLACE VIEW system_tasks AS
                SELECT
                    {job},
                    {job_type},
                    {status},
                    {subscriber_id},
                    {task_type},
                    {id},
                    {attempts},
                    {max_attempts},
                    {run_at},
                    {last_error},
                    {lock_at},
                    {lock_by},
                    {done_at},
                    {priority}
                FROM {apalis_schema}.{apalis_table}
                WHERE {job_type} = '{SYSTEM_TASK_APALIS_NAME}'
                AND jsonb_path_exists({job}, '$.{task_type} ? (@.type() == "string")')"#,
      apalis_schema = ApalisSchema::Schema.to_string(),
      apalis_table = ApalisJobs::Table.to_string(),
      job = ApalisJobs::Job.to_string(),
      job_type = ApalisJobs::JobType.to_string(),
      status = ApalisJobs::Status.to_string(),
      subscriber_id = ApalisJobs::SubscriberId.to_string(),
      task_type = ApalisJobs::TaskType.to_string(),
      id = ApalisJobs::Id.to_string(),
      attempts = ApalisJobs::Attempts.to_string(),
      max_attempts = ApalisJobs::MaxAttempts.to_string(),
      run_at = ApalisJobs::RunAt.to_string(),
      last_error = ApalisJobs::LastError.to_string(),
      lock_at = ApalisJobs::LockAt.to_string(),
      lock_by = ApalisJobs::LockBy.to_string(),
      done_at = ApalisJobs::DoneAt.to_string(),
      priority = ApalisJobs::Priority.to_string(),
    ))
    .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let db = manager.get_connection();

    db.execute_unprepared("DROP VIEW IF EXISTS subscriber_tasks").await?;

    db.execute_unprepared("DROP VIEW IF EXISTS system_tasks").await?;

    db.execute_unprepared(&format!(
      r#"DROP TRIGGER IF EXISTS {SETUP_APALIS_JOBS_EXTRA_FOREIGN_KEYS_TRIGGER_NAME} ON {apalis_schema}.{apalis_table}"#,
      apalis_schema = ApalisSchema::Schema.to_string(),
      apalis_table = ApalisJobs::Table.to_string()
    ))
    .await?;

    db.execute_unprepared(&format!(
      r#"DROP FUNCTION IF EXISTS {apalis_schema}.{SETUP_APALIS_JOBS_EXTRA_FOREIGN_KEYS_FUNCTION_NAME}()"#,
      apalis_schema = ApalisSchema::Schema.to_string(),
    ))
    .await?;

    manager
      .alter_table(
        TableAlterStatement::new()
          .table((ApalisSchema::Schema, ApalisJobs::Table))
          .drop_foreign_key("fk_apalis_jobs_subscriber_id")
          .drop_foreign_key("fk_apalis_jobs_subscription_id")
          .drop_column(ApalisJobs::SubscriberId)
          .drop_column(ApalisJobs::SubscriptionId)
          .to_owned(),
      )
      .await?;

    Ok(())
  }
}
