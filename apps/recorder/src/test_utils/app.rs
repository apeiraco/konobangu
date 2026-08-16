use std::{fmt::Debug, sync::Arc};

use once_cell::sync::OnceCell;
use typed_builder::TypedBuilder;

use crate::{
  app::AppContextTrait,
  errors::RecorderResult,
  task::TaskConfig,
  test_utils::{
    crypto::build_testing_crypto_service,
    database::{TestingDatabaseServiceConfig, build_testing_database_service},
    media::build_testing_media_service,
    mikan::{MikanMockServer, build_testing_mikan_client},
    storage::build_testing_storage_service,
    task::build_testing_task_service,
  },
};

#[derive(TypedBuilder)]
#[builder(field_defaults(default, setter(strip_option)))]
pub struct TestingAppContext {
  logger: Option<crate::logger::LoggerService>,
  db: Option<crate::database::DatabaseService>,
  config: Option<crate::app::AppConfig>,
  cache: Option<crate::cache::CacheService>,
  mikan: Option<crate::extract::mikan::MikanClient>,
  auth: Option<crate::auth::AuthService>,
  graphql: Option<crate::graphql::GraphQLService>,
  storage: Option<crate::storage::StorageService>,
  crypto: Option<crate::crypto::CryptoService>,
  media: Option<crate::media::MediaService>,
  #[builder(default = Arc::new(OnceCell::new()), setter(!strip_option))]
  task: Arc<OnceCell<crate::task::TaskService>>,
  message: Option<crate::message::MessageService>,
  #[builder(default = Some(String::from(env!("CARGO_MANIFEST_DIR"))))]
  working_dir: Option<String>,
  #[builder(default = crate::app::Environment::Testing, setter(!strip_option))]
  environment: crate::app::Environment,
}

impl TestingAppContext {
  pub fn set_task(&self, task: crate::task::TaskService) {
    self.task.get_or_init(|| task);
  }

  pub async fn from_config(config: TestingAppContextConfig) -> RecorderResult<Arc<Self>> {
    let mikan_base_url = config.mikan_base_url.expect("mikan_base_url is required");
    let mikan_client = build_testing_mikan_client(mikan_base_url).await?;
    let db_service = build_testing_database_service(config.database_config.unwrap_or_default()).await?;
    let crypto_service = build_testing_crypto_service().await?;
    let storage_service = build_testing_storage_service().await?;
    let media_service = build_testing_media_service().await?;
    let app_ctx = Arc::new(
      TestingAppContext::builder()
        .mikan(mikan_client)
        .db(db_service)
        .crypto(crypto_service)
        .storage(storage_service)
        .media(media_service)
        .build(),
    );

    let task_service = build_testing_task_service(config.task_config, app_ctx.clone()).await?;

    app_ctx.set_task(task_service);

    Ok(app_ctx)
  }
}

impl Debug for TestingAppContext {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "UnitTestAppContext")
  }
}

impl AppContextTrait for TestingAppContext {
  fn logger(&self) -> &crate::logger::LoggerService {
    self.logger.as_ref().expect("should set logger")
  }

  fn db(&self) -> &crate::database::DatabaseService {
    self.db.as_ref().expect("should set db")
  }

  fn config(&self) -> &crate::app::AppConfig {
    self.config.as_ref().expect("should set config")
  }

  fn cache(&self) -> &crate::cache::CacheService {
    self.cache.as_ref().expect("should set cache")
  }

  fn mikan(&self) -> &crate::extract::mikan::MikanClient {
    self.mikan.as_ref().expect("should set mikan")
  }

  fn auth(&self) -> &crate::auth::AuthService {
    self.auth.as_ref().expect("should set auth")
  }

  fn graphql(&self) -> &crate::graphql::GraphQLService {
    self.graphql.as_ref().expect("should set graphql")
  }

  fn storage(&self) -> &crate::storage::StorageService {
    self.storage.as_ref().expect("should set storage")
  }

  fn environment(&self) -> &crate::app::Environment {
    &self.environment
  }

  fn working_dir(&self) -> &String {
    self.working_dir.as_ref().expect("should set working_dir")
  }

  fn crypto(&self) -> &crate::crypto::CryptoService {
    self.crypto.as_ref().expect("should set crypto")
  }

  fn task(&self) -> &crate::task::TaskService {
    self.task.get().expect("should set task")
  }

  fn message(&self) -> &crate::message::MessageService {
    self.message.as_ref().expect("should set message")
  }

  fn media(&self) -> &crate::media::MediaService {
    self.media.as_ref().expect("should set media")
  }
}

#[derive(TypedBuilder, Debug)]
#[builder(field_defaults(default, setter(strip_option)))]
pub struct TestingAppContextConfig {
  pub mikan_base_url: Option<String>,
  pub database_config: Option<TestingDatabaseServiceConfig>,
  pub task_config: Option<TaskConfig>,
}

#[derive(TypedBuilder)]
pub struct TestingPreset {
  pub mikan_server: MikanMockServer,
  pub app_ctx: Arc<dyn AppContextTrait>,
}

impl TestingPreset {
  pub async fn default_with_config(config: TestingAppContextConfig) -> RecorderResult<Self> {
    let mikan_server = MikanMockServer::new().await?;

    let mixed_config = TestingAppContextConfig {
      mikan_base_url: Some(mikan_server.base_url().to_string()),
      ..config
    };

    let app_ctx = TestingAppContext::from_config(mixed_config).await?;

    let preset = Self::builder().mikan_server(mikan_server).app_ctx(app_ctx).build();
    Ok(preset)
  }

  pub async fn default() -> RecorderResult<Self> {
    Self::default_with_config(TestingAppContextConfig {
      mikan_base_url: None,
      database_config: None,
      task_config: None,
    })
    .await
  }
}
