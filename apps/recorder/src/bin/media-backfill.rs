//! Explicit public-only maintenance. GET never schedules work.
use std::sync::Arc;

use clap::Parser;
use futures::TryStreamExt;
use recorder::{
  app::{AppConfig, AppContext, Environment},
  errors::RecorderResult,
  media::{MediaService, derivative},
  storage::StorageService,
};
#[derive(Parser)]
struct Args {
  #[arg(long)]
  config_file: Option<String>,
  #[arg(long)]
  dotenv_file: Option<String>,
  #[arg(long, default_value = "production")]
  environment: Environment,
  #[arg(long)]
  write: bool,
  #[arg(long, default_value_t = 100)]
  limit: usize,
  #[arg(long, default_value_t = 0)]
  offset: usize,
}
async fn run_main() -> RecorderResult<()> {
  let args = Args::parse();
  if args.limit == 0 || args.limit > 10000 {
    return Err(recorder::media::invalid_options("Backfill limit must be between 1 and 10000"));
  }
  AppConfig::load_dotenv(&args.environment, args.dotenv_file.as_deref()).await?;
  let config = AppConfig::load_config(&args.environment, args.config_file.as_deref()).await?;
  let storage = StorageService::from_config(config.storage.clone()).await?;
  let media = MediaService::from_config(config.media.clone()).await?;
  let entries = storage.operator.lister_with("public/").recursive(true).await?.try_collect::<Vec<_>>().await?;
  let mut paths = entries
    .into_iter()
    .map(|e| e.path().to_owned())
    .filter(|p| {
      std::path::Path::new(p)
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| media.is_legacy_image_format(s))
    })
    .collect::<Vec<_>>();
  paths.sort();
  let context: Option<Arc<AppContext>> = if args.write {
    Some(AppContext::new(args.environment, config, ".").await?)
  } else {
    None
  };
  for path in paths.into_iter().skip(args.offset).take(args.limit) {
    let plans = storage.missing_derivatives(&path, &media).await;
    if plans.is_empty() {
      continue;
    }
    if let Some(ctx) = &context {
      let count = derivative::enqueue_missing(ctx.as_ref(), &path).await?;
      println!("queued {count}: {path}");
    } else {
      println!("dry-run {}: {path}", plans.len());
    }
  }
  if let Some(ctx) = context {
    use recorder::app::AppContextTrait;
    ctx.task().queue_database()?.clone().close().await?;
    ctx.auth().runtime().close().await?;
    ctx.db().as_ref().clone().close().await?;
  }
  Ok(())
}

fn main() -> RecorderResult<()> {
  tokio::runtime::Builder::new_multi_thread().enable_all().build()?.block_on(run_main())
}
