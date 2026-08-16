//! Loopback test support: the production AppContext/router and the shared
//! PostgreSQL fixture.
use std::path::PathBuf;

use figment::{Figment, providers::Serialized};
use recorder::app::{App, AppConfig, AppContext, AppContextTrait, Environment};
use serde::Deserialize;
use tokio::{net::TcpListener, sync::watch};

#[path = "../../integration/rls/support.rs"]
mod support;

#[derive(Deserialize)]
struct FixtureConfig {
  issuer: String,
  client_secret: String,
  ports: [u16; 5],
  directory: PathBuf,
}

async fn run_main() -> Result<(), Box<dyn std::error::Error>> {
  tokio::task::LocalSet::new().run_until(run()).await
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let config_path = std::env::var("KONOBANGU_AUTH_TEST_CONFIG")?;
  let fixture_config: FixtureConfig = serde_json::from_slice(&std::fs::read(config_path)?)?;
  let fixture = support::RlsFixture::new().await;
  use sea_orm::ConnectionTrait;
  // Make auth ID differ from subscriber ID to detect private-file identity
  // confusion.
  fixture
    .bootstrap
    .execute_unprepared("INSERT INTO auth_identity.auth (pid, auth_type, subscriber_id) VALUES ('fixture-extra-basic', 'basic', 1001)")
    .await?;
  let base_url = format!("http://127.0.0.1:{}", fixture_config.ports[0]);
  let working_dir = fixture_config.directory.to_string_lossy().into_owned();
  let mut db_config = fixture.bootstrap.config.clone();
  db_config.auto_migrate = true;
  db_config.max_connections = 5;
  let (shutdown, shutdown_rx) = watch::channel(false);
  let mut servers = Vec::new();
  let mut contexts = Vec::new();
  for (index, port) in fixture_config.ports.into_iter().enumerate() {
    let external_base_url = if index == 3 {
      format!("https://127.0.0.1:{port}")
    } else if index == 2 || index == 4 {
      format!("http://127.0.0.1:{port}")
    } else {
      base_url.clone()
    };
    let provider = if index == 2 {
      serde_json::json!({ "type": "basic", "username": "fixture-admin", "password": "fixture-admin-password" })
    } else {
      serde_json::json!({ "type": "oidc", "issuer": fixture_config.issuer, "audience": "konobangu-test", "client_id": "konobangu-test", "client_secret": fixture_config.client_secret })
    };
    let mut session = serde_json::json!({ "public_url": external_base_url, "cookie_secure": index == 3 });
    if index == 4 {
      session["idle_timeout_seconds"] = 1.into();
      session["absolute_timeout_seconds"] = 2.into();
      session["login_timeout_seconds"] = 1.into();
      session["cleanup_interval_seconds"] = 1.into();
    }
    // One instance also covers optional separate runtime credentials.
    let mut database = db_config.clone();
    let scheduler = if index == 1 {
      database.uri = fixture.app_uri.clone();
      database.auto_migrate = false;
      session["database_url"] = fixture.identity_uri.clone().into();
      serde_json::json!({"database": {"url": fixture.queue_uri}})
    } else {
      serde_json::json!({})
    };
    let auth = serde_json::json!({ "provider": provider, "session": session });
    let config: AppConfig = Figment::new()
      .merge(AppConfig::default_provider())
      .merge(Serialized::defaults(serde_json::json!({
          "database": database, "auth": auth, "scheduler": scheduler,
          "server": { "host": "127.0.0.1", "port": port, "middlewares": { "logger": { "enable": false } } },
          "logger": { "enable": false, "level": "off", "format": "compact" },
          "storage": { "data_dir": fixture_config.directory.join(format!("data-{index}")) }
      })))
      .extract()?;
    let context = AppContext::new(Environment::Testing, config, &working_dir).await?;
    for subscriber in [1, 2, 3] {
      let private = context.storage().build_subscriber_path(subscriber, "fixture-private.txt");
      context.storage().operator.write(private.as_ref(), "fixture-private-content").await?;
    }
    if index == 2 {
      fixture.bootstrap.execute_unprepared("INSERT INTO subscriptions (id,display_name,subscriber_id,source_url,enabled,category) VALUES (5000,'Media browser fixture',1,'https://example.test/media',true,'mikan_subscriber'); INSERT INTO bangumi (id,bangumi_type,subscriber_id,display_name,origin_name,season,poster_link) VALUES (5010,'mikan',1,'Media browser poster','Media browser poster',1,'/public/browser-poster.png'); INSERT INTO subscription_bangumi (subscriber_id,subscription_id,bangumi_id) VALUES (1,5000,5010)").await?;
      fixture
        .bootstrap
        .execute_unprepared(
          "INSERT INTO cron (id,subscriber_id,subscription_id,cron_expr,cron_timezone,next_run,last_run,locked_at,status,enabled,subscriber_task_cron) VALUES \
           (5020,1,5000,'0 */5 * * * \
           *','America/New_York','2026-10-04T19:38:05.559928Z','2026-10-04T19:38:05.559928Z','2026-10-04T19:38:05.559928Z','disabled',false,'{\"task_type\":\"\
           sync_one_subscription_sources\",\"subscription_id\":5000,\"subscriber_id\":1}'::jsonb)",
        )
        .await?;
      let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(256, 256, |x, y| image::Rgba([255, y as u8, 0, x as u8])));
      let mut bytes = std::io::Cursor::new(Vec::new());
      image.write_to(&mut bytes, image::ImageFormat::Png)?;
      for source in ["/public/browser-poster.png", "/subscribers/1/browser-poster.png"] {
        context.storage().write(source, bytes::Bytes::from(bytes.get_ref().clone())).await?;
        for plan in context.media().derivative_plans() {
          let fingerprint = recorder::media::derivative::fingerprint(&context.storage().stat(source).await?).unwrap();
          let mut task = recorder::task::OptimizeImageTask::builder()
            .source_path(source.into())
            .target_path("fixture".into())
            .format_options(plan.options)
            .derivation_version(Some(1))
            .source_fingerprint(Some(fingerprint))
            .build();
          if source.starts_with("/subscribers/") {
            task.subscriber_id = Some(1);
          }
          use recorder::task::AsyncTaskTrait;
          task.run_async(context.clone()).await?;
        }
      }
    }
    let network_log = fixture_config.directory.join("media-network.jsonl");
    let router = App::router(context.clone()).await?.layer(axum::middleware::from_fn(move |request: axum::extract::Request, next: axum::middleware::Next| {
      let network_log = network_log.clone();
      async move {
        let path = request.uri().to_string();
        let headers = request.headers();
        let accept = headers.get("accept").and_then(|v| v.to_str().ok()).unwrap_or("").to_owned();
        let ua = headers.get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("").to_owned();
        let response = next.run(request).await;
        if path.contains("browser-poster.png") {
          use std::io::Write;
          let mut log = std::fs::OpenOptions::new().create(true).append(true).open(network_log).expect("fixture network log");
          writeln!(log,"{}",serde_json::json!({"path":path,"accept":accept,"ua":ua,"status":response.status().as_u16(),"content_type":response.headers().get("content-type").and_then(|v|v.to_str().ok()),"content_length":response.headers().get("content-length").and_then(|v|v.to_str().ok()),"etag":response.headers().get("etag").and_then(|v|v.to_str().ok())})).expect("fixture network record");
        }
        response
      }
    }));
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await?;
    let mut cleaner_receiver = shutdown_rx.clone();
    let cleaner_context = context.clone();
    servers.push(tokio::spawn(async move {
      tokio::select! { _ = cleaner_context.auth().runtime().cleanup_loop() => {}, _ = cleaner_receiver.changed() => {} };
      Ok::<_, std::io::Error>(())
    }));
    let mut receiver = shutdown_rx.clone();
    servers.push(tokio::spawn(async move {
      axum::serve(listener, router.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .with_graceful_shutdown(async move {
          let _ = receiver.changed().await;
        })
        .await
    }));
    if index == 0 {
      let worker_context = context.clone();
      let mut stopped = shutdown_rx.clone();
      servers.push(tokio::task::spawn_local(async move {
        worker_context
          .task()
          .run_with_signal(Some(|| async move {
            let _ = stopped.changed().await;
          }))
          .await
          .map_err(|_| std::io::Error::other("Task fixture worker failed"))
      }));
    }
    contexts.push(context);
  }
  std::fs::write(
    fixture_config.directory.join("server.json"),
    serde_json::to_vec(&serde_json::json!({
        "baseUrl": base_url,
        "instanceUrl": format!("http://127.0.0.1:{}", fixture_config.ports[1]),
        "basicUrl": format!("http://127.0.0.1:{}", fixture_config.ports[2]),
        "secureUrl": format!("http://127.0.0.1:{}", fixture_config.ports[3]),
        "expiresUrl": format!("http://127.0.0.1:{}", fixture_config.ports[4]),
        "issuer": fixture_config.issuer
    }))?,
  )?;
  #[cfg(unix)]
  {
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! { result = tokio::signal::ctrl_c() => result?, _ = terminate.recv() => {} }
  }
  #[cfg(not(unix))]
  tokio::signal::ctrl_c().await?;
  shutdown.send(true)?;
  for server in servers {
    server.await??;
  }
  for context in contexts {
    context.db().as_ref().clone().close().await?;
    context.auth().runtime().close().await?;
    context.task().queue_database()?.clone().close().await?;
  }
  fixture.close().await;
  Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // LocalSet cleanup must use the current-thread testcontainers drop worker.
  tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(run_main())
}
