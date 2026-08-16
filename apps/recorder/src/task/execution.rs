//! Business fencing is independent of queue delivery and queue worker locks.
use std::{error::Error, fmt, time::Duration};

use apalis::prelude::*;
use sea_orm::{ConnectionTrait, DatabaseTransaction, DbBackend, DbErr, Statement};
use uuid::Uuid;

use crate::errors::RecorderError;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskEnvelope {
  pub task_id: String,
  pub generation: i64,
}

#[derive(Clone, Debug)]
pub struct TaskFence {
  pub envelope: TaskEnvelope,
  pub token: Uuid,
}

tokio::task_local! { pub static CURRENT_FENCE: TaskFence; }

pub async fn check(db: &DatabaseTransaction) -> Result<(), DbErr> {
  let Ok(fence) = CURRENT_FENCE.try_with(Clone::clone) else {
    return Ok(());
  };
  // A transaction-local tag lets the same ordinary role cancel only this
  // execution's in-flight SQL when dropping its future cannot stop the server.
  db.query_one_raw(Statement::from_sql_and_values(
    DbBackend::Postgres,
    "SELECT set_config('application_name', $1, true)",
    [format!("konobangu-task-{}", fence.token).into()],
  ))
  .await?;
  let present = db
    .query_one_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "SELECT id FROM task_runs WHERE id = $1 AND generation = $2 AND execution_token = $3 AND status = 'Running' AND execution_lease_until > \
       clock_timestamp() AND cancel_requested_at IS NULL AND archived_at IS NULL FOR UPDATE",
      [fence.envelope.task_id.into(), fence.envelope.generation.into(), fence.token.into()],
    ))
    .await?;
  if present.is_none() {
    return Err(DbErr::Custom("Task execution fence expired or was cancelled".into()));
  }
  Ok(())
}

/// SQLx future cancellation does not send a PostgreSQL CancelRequest. A
/// short-lived same-role control connection avoids borrowing the busy pool
/// (including max_connections=1), without new grants or a permanent pool.
pub async fn cancel_queries(db: &sea_orm::DatabaseConnection, token: Uuid) -> Result<(), DbErr> {
  use sqlx::Connection;
  let options = db.get_postgres_connection_pool().connect_options();
  let cancel = async {
    let mut connection = sqlx::PgConnection::connect_with(&options).await?;
    let result = sqlx::query_scalar::<_, bool>(
      "SELECT pg_cancel_backend(pid) FROM pg_stat_activity WHERE usename=current_user AND application_name=$1 AND state='active' AND pid<>pg_backend_pid()",
    )
    .bind(format!("konobangu-task-{token}"))
    .fetch_all(&mut connection)
    .await;
    let closed = connection.close().await;
    result?;
    closed?;
    Ok::<_, sqlx::Error>(())
  };
  tokio::time::timeout(Duration::from_secs(5), cancel)
    .await
    .map_err(|_| DbErr::Custom("Task SQL cancellation timed out".into()))?
    .map_err(|error| DbErr::Exec(sea_orm::RuntimeErr::SqlxError(error.into())))
}

#[derive(Debug)]
pub struct ExecutionError {
  pub retry_after: Option<Duration>,
  pub message: String,
}
impl fmt::Display for ExecutionError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.message)
  }
}
impl Error for ExecutionError {}
impl From<DbErr> for ExecutionError {
  fn from(error: DbErr) -> Self {
    let recoverable = recoverable(&RecorderError::from(error));
    Self {
      retry_after: recoverable.then_some(Duration::from_secs(5)),
      message: "Task database stage failed".into(),
    }
  }
}

/// Retry timing comes from the committed business attempt, never the queue's
/// delivery counter (which also includes duplicates and liveness recovery).
#[derive(Clone)]
pub struct BusinessRetryPolicy {
  pub shutdown: tokio::sync::watch::Receiver<bool>,
}
impl<Res> tower::retry::Policy<Task<TaskEnvelope>, Res, BoxDynError> for BusinessRetryPolicy {
  type Future = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>;
  fn retry(&mut self, _request: &mut Task<TaskEnvelope>, result: &mut Result<Res, BoxDynError>) -> Option<Self::Future> {
    if *self.shutdown.borrow() {
      return None;
    }
    let delay = result.as_ref().err()?.downcast_ref::<ExecutionError>()?.retry_after?;
    let mut shutdown = self.shutdown.clone();
    Some(Box::pin(async move {
      tokio::select! { _=tokio::time::sleep(delay)=>{}, _=shutdown.wait_for(|stopped|*stopped)=>{} }
    }))
  }

  fn clone_request(&mut self, request: &Task<TaskEnvelope>) -> Option<Task<TaskEnvelope>> {
    Some(request.clone())
  }
}

pub fn recoverable(error: &RecorderError) -> bool {
  // Transparent error wrappers may delegate source() past the wrapped error;
  // classify these typed boundaries before walking nested causes.
  match error {
    RecorderError::HttpResponseError { status, .. } => return status.is_server_error(),
    RecorderError::FetchError {
      source: fetch::FetchError::ReqwestError { source },
    } => return recoverable_http(source),
    RecorderError::FetchError {
      source: fetch::FetchError::RequestMiddlewareError {
        source: fetch::reqwest_middleware::Error::Reqwest(source),
      },
    } => return recoverable_http(source),
    RecorderError::HttpClientError {
      source: fetch::HttpClientError::ReqwestError { source },
    } => return recoverable_http(source),
    RecorderError::HttpClientError {
      source: fetch::HttpClientError::ReqwestMiddlewareError {
        source: fetch::reqwest_middleware::Error::Reqwest(source),
      },
    } => return recoverable_http(source),
    RecorderError::DbSqlxError { source } => return recoverable_sql(source),
    RecorderError::DbError { source } => match source {
      DbErr::Conn(sea_orm::RuntimeErr::SqlxError(sql))
      | DbErr::Exec(sea_orm::RuntimeErr::SqlxError(sql))
      | DbErr::Query(sea_orm::RuntimeErr::SqlxError(sql)) => return recoverable_sql(sql),
      DbErr::ConnectionAcquire(_) => return true,
      _ => {}
    },
    _ => {}
  }
  let mut cause: Option<&(dyn Error + 'static)> = Some(error);
  while let Some(current) = cause {
    if let Some(http) = current.downcast_ref::<fetch::reqwest::Error>() {
      return recoverable_http(http);
    }
    if let Some(sql) = current.downcast_ref::<sqlx::Error>() {
      return recoverable_sql(sql);
    }
    cause = current.source();
  }
  false
}

fn recoverable_http(error: &fetch::reqwest::Error) -> bool {
  error.is_timeout() || error.is_connect() || error.status().is_some_and(|status| status.is_server_error())
}
fn recoverable_sql(error: &sqlx::Error) -> bool {
  match error {
    sqlx::Error::Io(_) | sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed => true,
    sqlx::Error::Database(db) => db
      .code()
      .is_some_and(|code| code.starts_with("08") || matches!(code.as_ref(), "40001" | "40P01" | "57P01" | "57P02" | "57P03")),
    _ => false,
  }
}
