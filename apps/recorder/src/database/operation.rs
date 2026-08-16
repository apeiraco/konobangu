use std::{future::Future, pin::Pin, sync::Arc};

use sea_orm::{DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait};
use tokio::sync::Mutex;

use super::rls::bind_subscriber_to_transaction;

/// An operation owns completion; resolvers only borrow its transaction for
/// short DB work.
#[derive(Clone)]
pub struct IdentityOperation {
  transaction: Arc<Mutex<Option<DatabaseTransaction>>>,
  pub subscriber_id: i32,
}

impl IdentityOperation {
  pub async fn begin(db: &DatabaseConnection, subscriber_id: i32) -> Result<Self, DbErr> {
    if subscriber_id <= 0 {
      return Err(DbErr::Custom("A verified subscriber is required".into()));
    }
    let transaction = db.begin().await?;
    bind_subscriber_to_transaction(&transaction, subscriber_id).await?;
    Ok(Self {
      transaction: Arc::new(Mutex::new(Some(transaction))),
      subscriber_id,
    })
  }

  pub async fn run<T, E>(
    &self,
    operation: impl for<'a> FnOnce(&'a DatabaseTransaction) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'a>>,
  ) -> Result<T, E>
  where
    E: From<DbErr>,
  {
    let guard = self.transaction.lock().await;
    let transaction = guard.as_ref().ok_or_else(|| E::from(DbErr::Custom("Operation transaction is closed".into())))?;
    operation(transaction).await
  }

  pub async fn finish(&self, commit: bool) -> Result<(), DbErr> {
    let transaction = self
      .transaction
      .lock()
      .await
      .take()
      .ok_or_else(|| DbErr::Custom("Operation transaction is closed".into()))?;
    if commit { transaction.commit().await } else { transaction.rollback().await }
  }
}

/// Commit task database stages before proceeding to network or image work.
pub async fn begin_task_transaction(db: &DatabaseConnection, subscriber_id: i32) -> Result<DatabaseTransaction, DbErr> {
  if subscriber_id <= 0 {
    return Err(DbErr::Custom("A verified task owner is required".into()));
  }
  let transaction = db.begin().await?;
  bind_subscriber_to_transaction(&transaction, subscriber_id).await?;
  crate::task::execution::check(&transaction).await?;
  Ok(transaction)
}

/// PostgreSQL foreign-key checks bypass RLS, so task stages verify parent
/// visibility first.
pub async fn require_visible<E>(db: &DatabaseTransaction, id: i32) -> Result<(), DbErr>
where
  E: sea_orm::EntityTrait,
  E::PrimaryKey: sea_orm::PrimaryKeyTrait<ValueType = i32>,
{
  E::find_by_id(id)
    .one(db)
    .await?
    .ok_or_else(|| DbErr::RecordNotFound("Referenced resource is not available to this owner".into()))?;
  Ok(())
}

/// Revalidate immediately before each short business commit. The task row lock
/// prevents a concurrent token replacement while these writes commit.
pub async fn commit_task_transaction(transaction: DatabaseTransaction) -> Result<(), DbErr> {
  crate::task::execution::check(&transaction).await?;
  transaction.commit().await
}
