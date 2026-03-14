use std::{future::Future, ops::Deref, str::FromStr, sync::Arc};

use apalis::prelude::*;
use apalis_sql::{
    Config,
    context::SqlContext,
    postgres::{PgListen as ApalisPgListen, PostgresStorage as ApalisPostgresStorage},
};
use sea_orm::{ActiveModelTrait, sqlx::postgres::PgListener};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
    models::cron::{self, CRON_DUE_DEBUG_EVENT, CRON_DUE_EVENT},
    task::{
        AsyncTaskTrait, SUBSCRIBER_TASK_APALIS_NAME, SYSTEM_TASK_APALIS_NAME, SubscriberTask,
        TaskConfig,
        config::{default_subscriber_task_workers, default_system_task_workers},
        registry::SystemTask,
    },
};

pub struct TaskService {
    pub config: TaskConfig,
    ctx: Arc<dyn AppContextTrait>,
    subscriber_task_storage: Arc<RwLock<ApalisPostgresStorage<SubscriberTask>>>,
    system_task_storage: Arc<RwLock<ApalisPostgresStorage<SystemTask>>>,
    cron_worker_id: String,
}

impl TaskService {
    pub async fn from_config_and_ctx(
        mut config: TaskConfig,
        ctx: Arc<dyn AppContextTrait>,
    ) -> RecorderResult<Self> {
        if config.subscriber_task_concurrency == 0 {
            config.subscriber_task_concurrency = default_subscriber_task_workers();
        };
        if config.system_task_concurrency == 0 {
            config.system_task_concurrency = default_system_task_workers();
        };

        let pool = ctx.db().get_postgres_connection_pool().clone();
        let subscriber_task_storage_config = Config::new(SUBSCRIBER_TASK_APALIS_NAME)
            .set_reenqueue_orphaned_after(config.subscriber_task_reenqueue_orphaned_after);
        let system_task_storage_config = Config::new(SYSTEM_TASK_APALIS_NAME)
            .set_reenqueue_orphaned_after(config.system_task_reenqueue_orphaned_after);
        let subscriber_task_storage =
            ApalisPostgresStorage::new_with_config(pool.clone(), subscriber_task_storage_config);
        let system_task_storage =
            ApalisPostgresStorage::new_with_config(pool, system_task_storage_config);

        Ok(Self {
            config,
            cron_worker_id: Uuid::now_v7().to_string(),
            ctx,
            subscriber_task_storage: Arc::new(RwLock::new(subscriber_task_storage)),
            system_task_storage: Arc::new(RwLock::new(system_task_storage)),
        })
    }

    async fn run_subscriber_task(
        job: SubscriberTask,
        data: Data<Arc<dyn AppContextTrait>>,
    ) -> RecorderResult<()> {
        let ctx = data.deref().clone();

        job.run_async(ctx).await
    }

    async fn run_system_task(
        job: SystemTask,
        data: Data<Arc<dyn AppContextTrait>>,
    ) -> RecorderResult<()> {
        let ctx = data.deref().clone();
        job.run_async(ctx).await
    }

    pub async fn retry_subscriber_task(&self, job_id: String) -> RecorderResult<()> {
        {
            let mut storage = self.subscriber_task_storage.write().await;
            let task_id =
                TaskId::from_str(&job_id).map_err(|err| RecorderError::InvalidTaskId {
                    message: err.to_string(),
                })?;
            let worker_id = WorkerId::new(SUBSCRIBER_TASK_APALIS_NAME);
            storage.retry(&worker_id, &task_id).await?;
        }
        Ok(())
    }

    pub async fn retry_system_task(&self, job_id: String) -> RecorderResult<()> {
        {
            let mut storage = self.system_task_storage.write().await;
            let task_id =
                TaskId::from_str(&job_id).map_err(|err| RecorderError::InvalidTaskId {
                    message: err.to_string(),
                })?;
            let worker_id = WorkerId::new(SYSTEM_TASK_APALIS_NAME);
            storage.retry(&worker_id, &task_id).await?;
        }
        Ok(())
    }

    pub async fn add_subscriber_task(
        &self,
        subscriber_task: SubscriberTask,
    ) -> RecorderResult<TaskId> {
        let task_id = {
            let mut storage = self.subscriber_task_storage.write().await;
            let sql_context = {
                let mut c = SqlContext::default();
                c.set_max_attempts(1);
                c
            };
            let request = Request::new_with_ctx(subscriber_task, sql_context);
            storage.push_request(request).await?.task_id
        };

        Ok(task_id)
    }

    pub async fn add_system_task(&self, system_task: SystemTask) -> RecorderResult<TaskId> {
        let task_id = {
            let mut storage = self.system_task_storage.write().await;
            let sql_context = {
                let mut c = SqlContext::default();
                c.set_max_attempts(1);
                c
            };
            let request = Request::new_with_ctx(system_task, sql_context);
            storage.push_request(request).await?.task_id
        };

        Ok(task_id)
    }

    pub async fn add_subscriber_task_cron(
        &self,
        cm: cron::ActiveModel,
    ) -> RecorderResult<cron::Model> {
        let db = self.ctx.db();
        let m = cm.insert(db).await?;
        Ok(m)
    }

    pub async fn add_system_task_cron(&self, cm: cron::ActiveModel) -> RecorderResult<cron::Model> {
        let db = self.ctx.db();
        let m = cm.insert(db).await?;
        Ok(m)
    }

    pub async fn run(&self) -> RecorderResult<()> {
        self.run_with_signal(None::<fn() -> std::future::Ready<()>>)
            .await
    }

    pub async fn run_with_signal<F, Fut>(&self, shutdown_signal: Option<F>) -> RecorderResult<()>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send,
    {
        tokio::select! {
            _ =  {
                let monitor = self.setup_apalis_monitor().await?;
                async move {
                    if let Some(shutdown_signal) = shutdown_signal {
                        monitor
                            .run_with_signal(async move {
                                shutdown_signal().await;
                                tracing::info!("apalis shutting down...");
                                Ok(())
                            })
                            .await?;
                    } else {
                        monitor.run().await?;
                    }
                    Ok::<_, RecorderError>(())
                }
            } => {}
            _ = {
                let listener = self.setup_apalis_listener().await?;
                async move {
                    if let Err(e) = listener.listen().await {
                        tracing::error!("Error listening to apalis: {e}");
                    }
                    Ok::<_, RecorderError>(())
                }
            } => {},
            _ = {
                let mut listener = self.setup_cron_due_listening().await?;
                let cron_worker_id = self.cron_worker_id.clone();
                let retry_duration =
                    chrono::Duration::milliseconds(self.config.cron_retry_duration.as_millis() as i64);
                let cron_interval_duration = self.config.cron_interval_duration;
                async move {
                    listener.listen_all([CRON_DUE_EVENT as &str, CRON_DUE_DEBUG_EVENT as &str]).await?;

                    tokio::join!(
                        {
                            let ctx = self.ctx.clone();
                            async move {
                                if let Err(e) =
                                    Self::listen_cron_due(listener, ctx, &cron_worker_id, retry_duration)
                                        .await
                                {
                                    tracing::error!("Error listening to cron due: {e}");
                                }
                            }
                        },
                        {
                            let ctx = self.ctx.clone();
                            let mut interval = tokio::time::interval(cron_interval_duration);
                            async move {
                                loop {
                                    interval.tick().await;
                                    if let Err(e) = cron::Model::check_and_cleanup_expired_cron_locks(
                                        ctx.as_ref(),
                                        retry_duration,
                                    )
                                    .await
                                    {
                                        tracing::error!(
                                            "Error checking and cleaning up expired cron locks: {e}"
                                        );
                                    }

                                    if let Err(e) =
                                        cron::Model::check_and_trigger_due_crons(ctx.as_ref()).await
                                    {
                                        tracing::error!("Error checking and triggering due crons: {e}");
                                    }
                                }
                            }
                        }
                    );
                    Ok::<_, RecorderError>(())
                }
            } => {}
        };

        Ok(())
    }

    async fn setup_apalis_monitor(&self) -> RecorderResult<Monitor> {
        let mut apalis_monitor = Monitor::new();

        {
            let subscriber_task_worker = WorkerBuilder::new(SUBSCRIBER_TASK_APALIS_NAME)
                .concurrency(self.config.subscriber_task_concurrency as usize)
                .catch_panic()
                .enable_tracing()
                .data(self.ctx.clone())
                .backend({
                    let storage = self.subscriber_task_storage.read().await;
                    storage.clone()
                })
                .build_fn(Self::run_subscriber_task);

            let system_task_worker = WorkerBuilder::new(SYSTEM_TASK_APALIS_NAME)
                .concurrency(self.config.system_task_concurrency as usize)
                .catch_panic()
                .enable_tracing()
                .data(self.ctx.clone())
                .backend(self.system_task_storage.read().await.clone())
                .build_fn(Self::run_system_task);

            apalis_monitor = apalis_monitor
                .register(subscriber_task_worker)
                .register(system_task_worker);
        }

        Ok(apalis_monitor)
    }

    async fn setup_apalis_listener(&self) -> RecorderResult<ApalisPgListen> {
        let pool = self.ctx.db().get_postgres_connection_pool().clone();
        let mut apalis_pg_listener = ApalisPgListen::new(pool).await?;

        {
            let mut subscriber_task_storage = self.subscriber_task_storage.write().await;
            apalis_pg_listener.subscribe_with(&mut subscriber_task_storage);
        }

        {
            let mut system_task_storage = self.system_task_storage.write().await;
            apalis_pg_listener.subscribe_with(&mut system_task_storage);
        }

        Ok(apalis_pg_listener)
    }

    async fn setup_cron_due_listening(&self) -> RecorderResult<PgListener> {
        let pool = self.ctx.db().get_postgres_connection_pool().clone();
        let listener = PgListener::connect_with(&pool).await?;
        tracing::debug!("Cron due listener connected to postgres");

        Ok(listener)
    }

    async fn listen_cron_due(
        mut listener: PgListener,
        ctx: Arc<dyn AppContextTrait>,
        worker_id: &str,
        retry_duration: chrono::Duration,
    ) -> RecorderResult<()> {
        loop {
            let notification = listener.recv().await?;
            if notification.channel() == CRON_DUE_DEBUG_EVENT {
                tracing::debug!("Received cron due debug event: {:?}", notification);
                continue;
            } else if notification.channel() == CRON_DUE_EVENT
                && let Err(e) = cron::Model::handle_cron_notification(
                    ctx.as_ref(),
                    notification,
                    worker_id,
                    retry_duration,
                )
                .await
            {
                tracing::error!("Error handling cron notification: {e}");
            }
        }
    }
}

#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    use std::time::Duration;

    use chrono::Utc;
    use sea_orm::ActiveValue;
    use serial_test::serial;

    use super::*;
    use crate::{
        models::cron,
        task::EchoTask,
        test_utils::{
            app::{TestingAppContextConfig, TestingPreset},
            tracing::{logs_contain, setup_traced_test},
        },
    };

    #[tokio::test]
    #[serial]
    async fn test_check_and_trigger_due_crons_with_certain_interval() -> RecorderResult<()> {
        let _guard = setup_traced_test();

        let preset = TestingPreset::default_with_config(
            TestingAppContextConfig::builder()
                .task_config(TaskConfig {
                    cron_interval_duration: Duration::from_millis(1500),
                    ..Default::default()
                })
                .build(),
        )
        .await?;
        let app_ctx = preset.app_ctx;
        let task_service = app_ctx.task();

        let task_id = Uuid::now_v7().to_string();

        let echo_cron = cron::ActiveModel {
            cron_expr: ActiveValue::Set("*/1 * * * * *".to_string()),
            cron_timezone: ActiveValue::Set("Asia/Singapore".to_string()),
            system_task_cron: ActiveValue::Set(Some(
                EchoTask::builder().task_id(task_id.clone()).build().into(),
            )),
            ..Default::default()
        };

        task_service.add_system_task_cron(echo_cron).await?;

        task_service
            .run_with_signal(Some(async move || {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }))
            .await?;

        assert!(logs_contain(&format!(
            "EchoTask {task_id} start running at"
        )));

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_trigger_due_cron_when_mutating() -> RecorderResult<()> {
        let _guard = setup_traced_test();

        let preset = TestingPreset::default().await?;
        let app_ctx = preset.app_ctx;
        let task_service = app_ctx.task();

        let task_id = Uuid::now_v7().to_string();

        let echo_cron = cron::ActiveModel {
            cron_expr: ActiveValue::Set("* * * */1 * *".to_string()),
            cron_timezone: ActiveValue::Set("Asia/Singapore".to_string()),
            next_run: ActiveValue::Set(Some(Utc::now() + chrono::Duration::seconds(-10))),
            system_task_cron: ActiveValue::Set(Some(
                EchoTask::builder().task_id(task_id.clone()).build().into(),
            )),
            ..Default::default()
        };

        let task_runner = task_service.run_with_signal(Some(async move || {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }));

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        task_service.add_system_task_cron(echo_cron).await?;

        task_runner.await?;

        assert!(logs_contain(&format!(
            "EchoTask {task_id} start running at"
        )));

        Ok(())
    }
}

