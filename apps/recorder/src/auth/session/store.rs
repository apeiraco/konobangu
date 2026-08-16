use async_trait::async_trait;
use tower_sessions::{
  SessionStore,
  session::{Id, Record},
  session_store,
};
use tower_sessions_core_014::{SessionStore as LegacySessionStore, session as legacy};
use tower_sessions_sqlx_store::PostgresStore;

/// Bridges the store's core 0.14 records without duplicating its SQL or
/// encoding.
#[derive(Debug, Clone)]
pub struct PersistentSessionStore(pub PostgresStore);

fn to_legacy(record: &Record) -> legacy::Record {
  legacy::Record {
    id: legacy::Id(record.id.0),
    data: record.data.clone(),
    expiry_date: record.expiry_date,
  }
}

fn from_legacy(record: legacy::Record) -> Record {
  Record {
    id: Id(record.id.0),
    data: record.data,
    expiry_date: record.expiry_date,
  }
}

fn store_error(error: tower_sessions_core_014::session_store::Error) -> session_store::Error {
  session_store::Error::Backend(error.to_string())
}

#[async_trait]
impl SessionStore for PersistentSessionStore {
  async fn create(&self, record: &mut Record) -> session_store::Result<()> {
    let mut converted = to_legacy(record);
    self.0.create(&mut converted).await.map_err(store_error)?;
    // The backend can rotate a colliding ID during creation.
    record.id = Id(converted.id.0);
    Ok(())
  }

  async fn save(&self, record: &Record) -> session_store::Result<()> {
    self.0.save(&to_legacy(record)).await.map_err(store_error)
  }

  async fn load(&self, id: &Id) -> session_store::Result<Option<Record>> {
    self.0.load(&legacy::Id(id.0)).await.map(|record| record.map(from_legacy)).map_err(store_error)
  }

  async fn delete(&self, id: &Id) -> session_store::Result<()> {
    self.0.delete(&legacy::Id(id.0)).await.map_err(store_error)
  }
}
