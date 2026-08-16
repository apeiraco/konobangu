use recorder::{app::AppBuilder, database::DatabaseService, errors::RecorderResult};

#[tokio::main]
async fn main() -> RecorderResult<()> {
  let builder = AppBuilder::from_main_cli(None).await?;

  builder.load_env().await?;
  let mut database_config = builder.load_config().await?.database;
  database_config.auto_migrate = false;

  let database_service = DatabaseService::from_migration_config(database_config).await?;

  database_service.migrate_down().await?;

  Ok(())
}
