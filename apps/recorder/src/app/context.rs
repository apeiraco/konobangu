use std::{fmt::Debug, sync::Arc};

use tokio::sync::OnceCell;

use super::{Environment, config::AppConfig};
use crate::{
  auth::AuthService, cache::CacheService, crypto::CryptoService, database::DatabaseService, errors::RecorderResult, extract::mikan::MikanClient,
  graphql::GraphQLService, logger::LoggerService, media::MediaService, message::MessageService, storage::StorageService, task::TaskService,
};

pub trait AppContextTrait: Send + Sync + Debug {
  fn logger(&self) -> &LoggerService;
  fn db(&self) -> &DatabaseService;
  fn config(&self) -> &AppConfig;
  fn cache(&self) -> &CacheService;
  fn mikan(&self) -> &MikanClient;
  fn auth(&self) -> &AuthService;
  fn graphql(&self) -> &GraphQLService;
  fn storage(&self) -> &StorageService;
  fn working_dir(&self) -> &String;
  fn environment(&self) -> &Environment;
  fn crypto(&self) -> &CryptoService;
  fn task(&self) -> &TaskService;
  fn message(&self) -> &MessageService;
  fn media(&self) -> &MediaService;
}

pub struct AppContext {
  logger: LoggerService,
  db: DatabaseService,
  config: AppConfig,
  cache: CacheService,
  mikan: MikanClient,
  auth: AuthService,
  storage: StorageService,
  crypto: CryptoService,
  working_dir: String,
  environment: Environment,
  message: MessageService,
  media: MediaService,
  task: OnceCell<TaskService>,
  graphql: OnceCell<GraphQLService>,
}

impl AppContext {
  pub async fn new(environment: Environment, mut config: AppConfig, working_dir: impl ToString) -> RecorderResult<Arc<Self>> {
    config.resolve_database_urls();
    let config_cloned = config.clone();

    let logger = LoggerService::from_config(config.logger).await?;
    let cache = CacheService::from_config(config.cache).await?;
    let db = DatabaseService::from_app_config(&config_cloned).await?;
    let storage = StorageService::from_config(config.storage).await?;
    let message = MessageService::from_config(config.message).await?;
    let mikan = MikanClient::from_config(config.mikan).await?;
    let crypto = CryptoService::from_config(config.crypto).await?;
    let media = MediaService::from_config(config.media).await?;
    let auth = AuthService::from_conf(config.auth).await?;

    let ctx = Arc::new(AppContext {
      config: config_cloned,
      environment,
      logger,
      auth,
      cache,
      db,
      storage,
      mikan,
      working_dir: working_dir.to_string(),
      crypto,
      message,
      media,
      task: OnceCell::new(),
      graphql: OnceCell::new(),
    });

    let initialized = async {
      ctx
        .task
        .get_or_try_init(async || TaskService::from_config_and_ctx(config.task, ctx.clone()).await)
        .await?;

      ctx
        .graphql
        .get_or_try_init(async || GraphQLService::from_config_and_ctx(config.graphql, ctx.clone()).await)
        .await?;

      Ok::<(), crate::errors::RecorderError>(())
    }
    .await;
    if let Err(error) = initialized {
      ctx.auth.runtime().close().await?;
      if let Some(task) = ctx.task.get() {
        task.close().await?;
      }
      ctx.media.shutdown().await?;
      ctx.db.as_ref().clone().close().await?;
      return Err(error);
    }

    Ok(ctx)
  }
}

impl Debug for AppContext {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "AppContext")
  }
}

impl AppContextTrait for AppContext {
  fn logger(&self) -> &LoggerService {
    &self.logger
  }
  fn db(&self) -> &DatabaseService {
    &self.db
  }
  fn config(&self) -> &AppConfig {
    &self.config
  }
  fn cache(&self) -> &CacheService {
    &self.cache
  }
  fn mikan(&self) -> &MikanClient {
    &self.mikan
  }
  fn auth(&self) -> &AuthService {
    &self.auth
  }
  fn graphql(&self) -> &GraphQLService {
    self.graphql.get().expect("graphql should be set")
  }
  fn storage(&self) -> &StorageService {
    &self.storage
  }
  fn working_dir(&self) -> &String {
    &self.working_dir
  }
  fn environment(&self) -> &Environment {
    &self.environment
  }
  fn crypto(&self) -> &CryptoService {
    &self.crypto
  }
  fn task(&self) -> &TaskService {
    self.task.get().expect("task should be set")
  }
  fn message(&self) -> &MessageService {
    &self.message
  }
  fn media(&self) -> &MediaService {
    &self.media
  }
}
