use async_trait::async_trait;
use sea_orm::ActiveEnum;
use sea_orm_migration::{prelude::*, schema::*};

use crate::{
  migrations::defs::{ApalisJobs, ApalisSchema, Cron, CustomSchemaManagerExt, GeneralIds, Subscribers, Subscriptions, table_auto_z},
  models::cron::{
    CHECK_AND_TRIGGER_DUE_CRONS_FUNCTION_NAME, CRON_DUE_DEBUG_EVENT, CRON_DUE_EVENT, CronStatus, CronStatusEnum, NOTIFY_DUE_CRON_WHEN_MUTATING_FUNCTION_NAME,
    NOTIFY_DUE_CRON_WHEN_MUTATING_TRIGGER_NAME, SETUP_CRON_EXTRA_FOREIGN_KEYS_FUNCTION_NAME, SETUP_CRON_EXTRA_FOREIGN_KEYS_TRIGGER_NAME,
  },
  task::{SETUP_APALIS_JOBS_EXTRA_FOREIGN_KEYS_FUNCTION_NAME, SUBSCRIBER_TASK_APALIS_NAME, SYSTEM_TASK_APALIS_NAME},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    create_postgres_enum_for_active_enum!(
      manager,
      CronStatusEnum,
      CronStatus::Pending,
      CronStatus::Running,
      CronStatus::Completed,
      CronStatus::Failed,
      CronStatus::Disabled
    )
    .await?;

    manager
      .create_table(
        table_auto_z(Cron::Table)
          .col(pk_auto(Cron::Id))
          .col(string(Cron::CronExpr))
          .col(string(Cron::CronTimezone))
          .col(integer_null(Cron::SubscriberId))
          .col(integer_null(Cron::SubscriptionId))
          .col(timestamp_with_time_zone_null(Cron::NextRun))
          .col(timestamp_with_time_zone_null(Cron::LastRun))
          .col(string_null(Cron::LastError))
          .col(boolean(Cron::Enabled).default(true))
          .col(string_null(Cron::LockedBy))
          .col(timestamp_with_time_zone_null(Cron::LockedAt))
          .col(integer_null(Cron::TimeoutMs).default(5000))
          .col(integer(Cron::Attempts).default(0))
          .col(integer(Cron::MaxAttempts).default(1))
          .col(integer(Cron::Priority).default(0))
          .col(enumeration(Cron::Status, CronStatusEnum, CronStatus::iden_values()).default(CronStatus::Pending))
          .col(json_binary_null(Cron::SubscriberTaskCron))
          .col(json_binary_null(Cron::SystemTaskCron))
          .foreign_key(
            ForeignKey::create()
              .name("fk_cron_subscriber_id")
              .from(Cron::Table, Cron::SubscriberId)
              .to(Subscribers::Table, Subscribers::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Restrict),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_cron_subscription_id")
              .from(Cron::Table, Cron::SubscriptionId)
              .to(Subscriptions::Table, Subscriptions::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Restrict),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_postgres_auto_update_ts_trigger_for_col(Cron::Table, GeneralIds::UpdatedAt)
      .await?;

    manager
      .create_index(
        IndexCreateStatement::new()
          .if_not_exists()
          .name("idx_cron_next_run")
          .table(Cron::Table)
          .col(Cron::NextRun)
          .to_owned(),
      )
      .await?;

    let db = manager.get_connection();

    db.execute_unprepared(&format!(
            r#"CREATE OR REPLACE FUNCTION {SETUP_CRON_EXTRA_FOREIGN_KEYS_FUNCTION_NAME}() RETURNS trigger AS $$
            DECLARE
                new_subscriber_task_subscriber_id integer;
                new_subscriber_task_subscription_id integer;
                new_system_task_subscriber_id integer;
            BEGIN
                new_subscriber_task_subscriber_id = (NEW.{subscriber_task_cron} ->> 'subscriber_id')::integer;
                new_subscriber_task_subscription_id = (NEW.{subscriber_task_cron} ->> 'subscription_id')::integer;
                new_system_task_subscriber_id = (NEW.{system_task_cron} ->> 'subscriber_id')::integer;
                IF new_subscriber_task_subscriber_id IS DISTINCT FROM (OLD.{subscriber_task_cron} ->> 'subscriber_id')::integer AND new_subscriber_task_subscriber_id IS DISTINCT FROM NEW.{subscriber_id} THEN
                    NEW.{subscriber_id} = new_subscriber_task_subscriber_id;
                END IF;
                IF new_subscriber_task_subscription_id IS DISTINCT FROM (OLD.{subscriber_task_cron} ->> 'subscription_id')::integer AND new_subscriber_task_subscription_id IS DISTINCT FROM NEW.{subscription_id} THEN
                    NEW.{subscription_id} = new_subscriber_task_subscription_id;
                END IF;
                IF new_system_task_subscriber_id IS DISTINCT FROM (OLD.{system_task_cron} ->> 'subscriber_id')::integer AND new_system_task_subscriber_id IS DISTINCT FROM NEW.{subscriber_id} THEN
                    NEW.{subscriber_id} = new_system_task_subscriber_id;
                END IF;
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;"#,
            subscriber_task_cron = Cron::SubscriberTaskCron.to_string(),
            subscriber_id = Cron::SubscriberId.to_string(),
            subscription_id = Cron::SubscriptionId.to_string(),
            system_task_cron = Cron::SystemTaskCron.to_string(),
        )).await?;

    db.execute_unprepared(&format!(
      r#"CREATE OR REPLACE TRIGGER {SETUP_CRON_EXTRA_FOREIGN_KEYS_TRIGGER_NAME}
                BEFORE INSERT OR UPDATE ON {table}
                FOR EACH ROW
                EXECUTE FUNCTION {SETUP_CRON_EXTRA_FOREIGN_KEYS_FUNCTION_NAME}();"#,
      table = Cron::Table.to_string(),
    ))
    .await?;

    db.execute_unprepared(&format!(
      r#"CREATE OR REPLACE FUNCTION {NOTIFY_DUE_CRON_WHEN_MUTATING_FUNCTION_NAME}() RETURNS trigger AS $$
        BEGIN
            -- Check if the cron is due to run
            IF NEW.{next_run} IS NOT NULL
               AND NEW.{next_run} <= CURRENT_TIMESTAMP
               AND NEW.{enabled} = true
               AND NEW.{status} = '{pending}'::{status_type}
               AND NEW.{attempts} < NEW.{max_attempts}
               -- Check if not locked or lock timeout
               AND (
                  NEW.{locked_at} IS NULL
                  OR (
                    NEW.{timeout_ms} IS NOT NULL
                    AND (NEW.{locked_at} + NEW.{timeout_ms} * INTERVAL '1 millisecond') <= CURRENT_TIMESTAMP
                  )
               )
               -- Make sure the cron is a new due event, not a repeat event
               AND (
                  OLD.{next_run} IS NULL
                  OR OLD.{next_run} > CURRENT_TIMESTAMP
                  OR OLD.{enabled} = false
                  OR OLD.{status} IS DISTINCT FROM '{pending}'
                  OR OLD.{attempts} IS DISTINCT FROM NEW.{attempts}
               )
               THEN
                  PERFORM pg_notify('{CRON_DUE_EVENT}', row_to_json(NEW)::text);
            END IF;
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;"#,
      next_run = Cron::NextRun.to_string(),
      enabled = Cron::Enabled.to_string(),
      locked_at = Cron::LockedAt.to_string(),
      timeout_ms = Cron::TimeoutMs.to_string(),
      status = Cron::Status.to_string(),
      pending = CronStatus::Pending.to_value(),
      attempts = Cron::Attempts.to_string(),
      max_attempts = Cron::MaxAttempts.to_string(),
      status_type = CronStatus::name(),
    ))
    .await?;

    db.execute_unprepared(&format!(
      r#"CREATE OR REPLACE TRIGGER {NOTIFY_DUE_CRON_WHEN_MUTATING_TRIGGER_NAME}
                AFTER INSERT OR UPDATE ON {table}
                FOR EACH ROW
                EXECUTE FUNCTION {NOTIFY_DUE_CRON_WHEN_MUTATING_FUNCTION_NAME}();"#,
      table = Cron::Table.to_string(),
    ))
    .await?;

    db.execute_unprepared(&format!(
            r#"CREATE OR REPLACE FUNCTION {CHECK_AND_TRIGGER_DUE_CRONS_FUNCTION_NAME}() RETURNS INTEGER AS $$
            DECLARE
                cron_record RECORD;
                notification_count INTEGER := 0;
            BEGIN
                FOR cron_record IN
                    SELECT * FROM {table}
                    WHERE {next_run} IS NOT NULL
                        AND {next_run} <= CURRENT_TIMESTAMP
                        AND {enabled} = true
                        AND {status} = '{pending}'::{status_type}
                        AND {attempts} < {max_attempts}
                        AND (
                            {locked_at} IS NULL
                            OR (
                                {timeout_ms} IS NOT NULL
                                AND {locked_at} + {timeout_ms} * INTERVAL '1 millisecond' <= CURRENT_TIMESTAMP
                            )
                        )
                    ORDER BY {priority} ASC, {next_run} ASC
                    FOR UPDATE SKIP LOCKED
                LOOP
                    -- PERFORM pg_notify('{CRON_DUE_DEBUG_EVENT}',format('Found due cron: value=%s; Now time: %s', row_to_json(cron_record)::text, CURRENT_TIMESTAMP));
                    PERFORM pg_notify('{CRON_DUE_EVENT}', row_to_json(cron_record)::text);
                    notification_count := notification_count + 1;
                END LOOP;

                -- PERFORM pg_notify('{CRON_DUE_DEBUG_EVENT}', format('Notification count: %I; Now time: %s', notification_count, CURRENT_TIMESTAMP));
                RETURN notification_count;
            END;
            $$ LANGUAGE plpgsql;"#,
            table = Cron::Table.to_string(),
            next_run = Cron::NextRun.to_string(),
            enabled = Cron::Enabled.to_string(),
            status = Cron::Status.to_string(),
            pending = CronStatus::Pending.to_value(),
            locked_at = Cron::LockedAt.to_string(),
            timeout_ms = Cron::TimeoutMs.to_string(),
            priority = Cron::Priority.to_string(),
            attempts = Cron::Attempts.to_string(),
            max_attempts = Cron::MaxAttempts.to_string(),
            status_type = CronStatus::name(),
        ))
        .await?;

    manager
      .alter_table(
        TableAlterStatement::new()
          .table((ApalisSchema::Schema, ApalisJobs::Table))
          .add_column_if_not_exists(integer_null(ApalisJobs::CronId))
          .add_foreign_key(
            TableForeignKey::new()
              .name("fk_apalis_jobs_cron_id")
              .from_tbl((ApalisSchema::Schema, ApalisJobs::Table))
              .from_col(ApalisJobs::CronId)
              .to_tbl(Cron::Table)
              .to_col(Cron::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Restrict),
          )
          .to_owned(),
      )
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
                        {subscription_id},
                        {cron_id}
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
      cron_id = ApalisJobs::CronId.to_string(),
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
                        {priority},
                        {cron_id}
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
      cron_id = ApalisJobs::CronId.to_string(),
    ))
    .await?;

    db.execute_unprepared(&format!(
      r#"
          UPDATE {apalis_schema}.{apalis_table} SET {cron_id} = ({job} ->> '{cron_id}')::integer
        "#,
      apalis_schema = ApalisSchema::Schema.to_string(),
      apalis_table = ApalisJobs::Table.to_string(),
      job = ApalisJobs::Job.to_string(),
      cron_id = ApalisJobs::CronId.to_string(),
    ))
    .await?;

    db.execute_unprepared(&format!(
            r#"CREATE OR REPLACE FUNCTION {apalis_schema}.{SETUP_APALIS_JOBS_EXTRA_FOREIGN_KEYS_FUNCTION_NAME}() RETURNS trigger AS $$
            DECLARE
                new_job_subscriber_id integer;
                new_job_subscription_id integer;
                new_job_cron_id integer;
                new_job_task_type text;
            BEGIN
                new_job_subscriber_id = (NEW.{job} ->> '{subscriber_id}')::integer;
                new_job_subscription_id = (NEW.{job} ->> '{subscription_id}')::integer;
                new_job_cron_id = (NEW.{job} ->> '{cron_id}')::integer;
                new_job_task_type = (NEW.{job} ->> '{task_type}')::text;
                IF new_job_subscriber_id IS DISTINCT FROM (OLD.{job} ->> '{subscriber_id}')::integer AND new_job_subscriber_id IS DISTINCT FROM NEW.{subscriber_id} THEN
                    NEW.{subscriber_id} = new_job_subscriber_id;
                END IF;
                IF new_job_subscription_id IS DISTINCT FROM (OLD.{job} ->> '{subscription_id}')::integer AND new_job_subscription_id IS DISTINCT FROM NEW.{subscription_id} THEN
                    NEW.{subscription_id} = new_job_subscription_id;
                END IF;
                IF new_job_cron_id IS DISTINCT FROM (OLD.{job} ->> '{cron_id}')::integer AND new_job_cron_id IS DISTINCT FROM NEW.{cron_id} THEN
                    NEW.{cron_id} = new_job_cron_id;
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
            cron_id = ApalisJobs::CronId.to_string(),
            task_type = ApalisJobs::TaskType.to_string(),
        )).await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let db = manager.get_connection();

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

    manager
      .alter_table(
        TableAlterStatement::new()
          .table((ApalisSchema::Schema, ApalisJobs::Table))
          .drop_column(ApalisJobs::CronId)
          .drop_foreign_key("fk_apalis_jobs_cron_id")
          .to_owned(),
      )
      .await?;

    db.execute_unprepared(&format!(
      r#"DROP TRIGGER IF EXISTS {NOTIFY_DUE_CRON_WHEN_MUTATING_TRIGGER_NAME} ON {table};"#,
      table = Cron::Table.to_string(),
    ))
    .await?;

    db.execute_unprepared(&format!(r#"DROP FUNCTION IF EXISTS {NOTIFY_DUE_CRON_WHEN_MUTATING_FUNCTION_NAME}();"#,))
      .await?;

    db.execute_unprepared(&format!(r#"DROP FUNCTION IF EXISTS {CHECK_AND_TRIGGER_DUE_CRONS_FUNCTION_NAME}();"#,))
      .await?;

    manager.drop_table(TableDropStatement::new().if_exists().table(Cron::Table).to_owned()).await?;

    manager.drop_postgres_enum_for_active_enum(CronStatusEnum).await?;

    Ok(())
  }
}
