use std::{
  any::{Any, TypeId},
  collections::HashMap,
  sync::{Arc, Mutex},
};

use async_graphql::dataloader::{DataLoader, HashMapCache, Loader};
use sea_orm::{DbErr, EntityTrait, QueryTrait, Select};

use crate::database::operation::IdentityOperation;

/// Every request owns its loaders; neither identity nor rows enter schema-wide
/// caches.
#[derive(Default)]
pub struct RequestLoaders(Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>);

struct RelationLoader<E: EntityTrait> {
  operation: IdentityOperation,
  statements: Mutex<HashMap<String, Select<E>>>,
}

impl<E: EntityTrait> Loader<String> for RelationLoader<E>
where
  E::Model: Sync,
{
  type Value = Option<E::Model>;
  type Error = Arc<DbErr>;
  async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
    let statements = {
      let mut pending = self.statements.lock().expect("request loader mutex poisoned");
      keys
        .iter()
        .filter_map(|key| pending.remove(key).map(|statement| (key.clone(), statement)))
        .collect::<Vec<_>>()
    };
    self
      .operation
      .run(move |db| {
        Box::pin(async move {
          let mut rows = HashMap::with_capacity(statements.len());
          for (key, statement) in statements {
            rows.insert(key, statement.one(db).await.map_err(Arc::new)?);
          }
          Ok::<_, Arc<DbErr>>(rows)
        })
      })
      .await
  }
}

impl RequestLoaders {
  pub async fn one<E: EntityTrait>(&self, operation: &IdentityOperation, statement: Select<E>) -> Result<Option<E::Model>, DbErr>
  where
    E::Model: Sync,
  {
    // SQL plus bound values distinguish filtered relations without storing
    // credentials in logs.
    let key = statement.build(sea_orm::DbBackend::Postgres).to_string();
    let loader = {
      let mut loaders = self.0.lock().expect("request loader registry mutex poisoned");
      loaders
        .entry(TypeId::of::<E>())
        .or_insert_with(|| {
          Arc::new(DataLoader::with_cache(
            RelationLoader::<E> {
              operation: operation.clone(),
              statements: Mutex::new(HashMap::new()),
            },
            tokio::spawn,
            HashMapCache::default(),
          ))
        })
        .clone()
        .downcast::<DataLoader<RelationLoader<E>, HashMapCache>>()
        .expect("entity loader type matches registry key")
    };
    loader
      .loader()
      .statements
      .lock()
      .expect("request loader mutex poisoned")
      .insert(key.clone(), statement);
    loader
      .load_one(key)
      .await
      .map(|row| row.flatten())
      .map_err(|error| DbErr::Custom(error.to_string()))
  }
}
