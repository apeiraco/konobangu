use recorder::{app::AppBuilder, errors::RecorderResult};

async fn run_main() -> RecorderResult<()> {
  if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("media-smoke")) {
    return recorder::media::smoke::run(std::iter::once(std::ffi::OsString::from("media-smoke")).chain(std::env::args_os().skip(2))).await;
  }
  let builder = AppBuilder::from_main_cli(None).await?;

  let app = builder.build().await?;

  app.serve().await?;

  Ok(())
}

fn main() -> RecorderResult<()> {
  tokio::runtime::Builder::new_multi_thread().enable_all().build()?.block_on(run_main())
}
