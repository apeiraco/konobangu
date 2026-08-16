use crate::{
  errors::RecorderResult,
  storage::{StorageConfig, StorageService},
};

pub async fn build_testing_storage_service() -> RecorderResult<StorageService> {
  let service = StorageService::from_config(StorageConfig {
    data_dir: "tests/data".to_string(),
  })
  .await?;

  Ok(service)
}
