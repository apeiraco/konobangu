use std::{
  panic::AssertUnwindSafe,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::Duration,
};

use async_graphql::{Request, dynamic::Schema};
use axum::{
  Router,
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  routing::get,
};
use fetch::reqwest;
use futures::FutureExt;
use recorder::{
  app::AppContextTrait,
  auth::{AuthConfig, AuthService, AuthUserInfo},
  database::{DatabaseService, operation::begin_task_transaction},
  graphql::{build_schema, operation::execute_operation},
  models::auth::{AuthType, Model},
  test_utils::{app::TestingAppContext, crypto::build_testing_crypto_service, mikan::build_testing_mikan_client},
  web::controller::{ControllerTrait, credential3rd},
};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use tokio::{
  sync::{Mutex, oneshot},
  task::JoinHandle,
};

use super::support::RlsFixture;

fn user(subscriber_id: i32) -> AuthUserInfo {
  AuthUserInfo {
    auth_type: AuthType::Basic,
    subscriber_auth: Model {
      id: subscriber_id,
      pid: "command-fixture".into(),
      issuer: None,
      subscriber_id,
      auth_type: AuthType::Basic,
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    },
  }
}
async fn create(schema: &Schema, ctx: &dyn AppContextTrait, owner: i32) -> i64 {
  let response = execute_operation(
    schema,
    ctx.db(),
    user(owner),
    Request::new(r#"mutation { credential3rdCreateOne(data: { credentialType: mikan, username: "fixture", password: "fixture" }) { id } }"#),
  )
  .await;
  assert!(response.errors.is_empty(), "{:?}", response.errors);
  response.data.into_json().unwrap()["credential3rdCreateOne"]["id"].as_i64().unwrap()
}
async fn cookie(fixture: &RlsFixture, id: i64) -> Option<String> {
  fixture
    .bootstrap
    .query_one_raw(Statement::from_sql_and_values(
      DbBackend::Postgres,
      "SELECT cookies FROM credential3rd WHERE id = $1",
      [id.into()],
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get_by_index(0)
    .unwrap()
}
fn command(client: &reqwest::Client, base: &str, id: i64) -> reqwest::RequestBuilder {
  client
    .post(format!("{base}/api/credential3rd/{id}/check-available"))
    .basic_auth("fixture-admin", Some("fixture-password"))
    .header("Origin", base)
    .header("X-Konobangu-CSRF", "1")
    .json(&json!({}))
}

struct SlowResponse {
  entered: oneshot::Sender<()>,
  release: oneshot::Receiver<()>,
}

#[derive(Default)]
struct Remote {
  calls: AtomicUsize,
  fail: std::sync::atomic::AtomicBool,
  slow: Mutex<Option<SlowResponse>>,
}

async fn account(State(remote): State<Arc<Remote>>) -> Response {
  remote.calls.fetch_add(1, Ordering::SeqCst);
  // Take the gate without holding a mutex while awaiting the coordinator.
  let slow = remote.slow.lock().await.take();
  let cookie = if let Some(slow) = slow {
    if slow.entered.send(()).is_err() {
      return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    match tokio::time::timeout(Duration::from_secs(45), slow.release).await {
      Ok(Ok(())) => "command_cookie=stale; Path=/; Max-Age=3600",
      Ok(Err(_)) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
      Err(_) => return StatusCode::GATEWAY_TIMEOUT.into_response(),
    }
  } else if remote.fail.load(Ordering::SeqCst) {
    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
  } else {
    "command_cookie=committed; Path=/; Max-Age=3600"
  };
  ([("set-cookie", cookie)], "").into_response()
}

struct HttpServer {
  base: String,
  shutdown: Option<oneshot::Sender<()>>,
  task: Option<JoinHandle<std::io::Result<()>>>,
}

impl HttpServer {
  fn start(listener: tokio::net::TcpListener, router: Router) -> Self {
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (shutdown, stopped) = oneshot::channel();
    let task = tokio::spawn(async move {
      axum::serve(listener, router)
        .with_graceful_shutdown(async {
          let _ = stopped.await;
        })
        .await
    });
    Self {
      base,
      shutdown: Some(shutdown),
      task: Some(task),
    }
  }

  async fn stop(&mut self) -> Result<(), String> {
    if let Some(shutdown) = self.shutdown.take() {
      let _ = shutdown.send(());
    }
    let Some(mut task) = self.task.take() else {
      return Ok(());
    };
    match tokio::time::timeout(Duration::from_secs(10), &mut task).await {
      Ok(Ok(result)) => result.map_err(|error| format!("HTTP server: {error}")),
      Ok(Err(error)) => Err(format!("HTTP task: {error}")),
      Err(_) => {
        task.abort();
        let _ = task.await;
        Err("HTTP shutdown watchdog expired; task aborted and joined".into())
      }
    }
  }
}

impl Drop for HttpServer {
  fn drop(&mut self) {
    // Explicit stop joins normal/failure paths; abort protects setup unwinding.
    if let Some(task) = self.task.take() {
      task.abort();
    }
  }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn credential_command_uses_short_stages_and_rejects_stale_writes() {
  let fixture = RlsFixture::new().await;
  let mut config = fixture.bootstrap.config.clone();
  config.uri = fixture.app_uri.clone();
  config.auto_migrate = false;
  config.max_connections = 1;
  let remote = Arc::new(Remote::default());
  let mut remote_server = HttpServer::start(
    tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap(),
    Router::new().route("/Account/Manage", get(account)).with_state(remote.clone()),
  );
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let base = format!("http://{}", listener.local_addr().unwrap());
  let auth: AuthConfig = serde_json::from_value(json!({
    "provider": {"type": "basic", "username": "fixture-admin", "password": "fixture-password"},
    "session": {"database_url": fixture.identity_uri, "public_url": base, "cookie_secure": false},
  }))
  .unwrap();
  let ctx: Arc<dyn AppContextTrait> = Arc::new(
    TestingAppContext::builder()
      .db(DatabaseService::from_config(config).await.unwrap())
      .crypto(build_testing_crypto_service().await.unwrap())
      .mikan(build_testing_mikan_client(&remote_server.base).await.unwrap())
      .auth(AuthService::from_conf(auth).await.unwrap())
      .build(),
  );
  let schema = build_schema(ctx.clone(), None, None).unwrap();
  let router = credential3rd::create(ctx.clone())
    .await
    .unwrap()
    .apply_to(Router::new())
    .with_state(ctx.clone())
    .layer(ctx.auth().runtime().layer());
  let mut server = HttpServer::start(listener, router);
  let client = reqwest::Client::builder().timeout(Duration::from_secs(60)).build().unwrap();
  let mut gate = None;
  let mut checking = None;
  // Catch primary assertion failures so they cannot bypass owned-task cleanup.
  let outcome = AssertUnwindSafe(async {
    assert!(!schema.sdl().contains("credential3rdCheckAvailable"));
    let own = create(&schema, ctx.as_ref(), 1).await;
    let other = create(&schema, ctx.as_ref(), 1002).await;
    assert_eq!(command(&client, &base, other).send().await.unwrap().status(), 403);
    assert_eq!(
      client
        .post(format!("{base}/api/credential3rd/{own}/check-available"))
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .status(),
      401
    );
    assert_eq!(
      command(&client, &base, own)
        .headers(reqwest::header::HeaderMap::from_iter([(
          reqwest::header::ORIGIN,
          "https://foreign.example".parse().unwrap(),
        )]))
        .send()
        .await
        .unwrap()
        .status(),
      403
    );
    assert_eq!(
      command(&client, &base, own)
        .headers(reqwest::header::HeaderMap::from_iter([(
          "x-konobangu-csrf".parse().unwrap(),
          "0".parse().unwrap(),
        )]))
        .send()
        .await
        .unwrap()
        .status(),
      403
    );
    assert_eq!(
      command(&client, &base, own)
        .headers(reqwest::header::HeaderMap::from_iter([(
          reqwest::header::CONTENT_TYPE,
          "text/plain".parse().unwrap(),
        )]))
        .send()
        .await
        .unwrap()
        .status(),
      415
    );
    assert_eq!(cookie(&fixture, own).await, None);
    assert_eq!(cookie(&fixture, other).await, None);
    assert_eq!(remote.calls.load(Ordering::SeqCst), 0);

    let response = command(&client, &base, own).send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert_eq!(response.json::<Value>().await.unwrap(), json!({"available": true}));
    let encrypted = cookie(&fixture, own).await.unwrap();
    assert!(!encrypted.contains("command_cookie"));
    assert!(ctx.crypto().decrypt_string(&encrypted).unwrap().contains("command_cookie"));
    assert_eq!(remote.calls.load(Ordering::SeqCst), 1);

    let failed = create(&schema, ctx.as_ref(), 1).await;
    remote.fail.store(true, Ordering::SeqCst);
    assert_eq!(command(&client, &base, failed).send().await.unwrap().status(), 502);
    assert_eq!(cookie(&fixture, failed).await, None);
    assert_eq!(remote.calls.load(Ordering::SeqCst), 2);
    remote.fail.store(false, Ordering::SeqCst);

    let stale = create(&schema, ctx.as_ref(), 1).await;
    let (notify_entered, entered) = oneshot::channel();
    let (release, allowed) = oneshot::channel();
    *remote.slow.lock().await = Some(SlowResponse {
      entered: notify_entered,
      release: allowed,
    });
    gate = Some(release);
    checking = Some(tokio::spawn(command(&client, &base, stale).send()));
    let concurrent: Result<(), String> = async {
      // Real synchronous credential decryption precedes the remote request.
      tokio::time::timeout(Duration::from_secs(30), entered)
        .await
        .map_err(|_| "credential preparation / external-entry deadline")?
        .map_err(|_| "external-entry notifier closed")?;
      // Only DB acquire/query/commit is measured against the strict two
      // seconds.
      tokio::time::timeout(Duration::from_secs(2), async {
        let transaction = begin_task_transaction(ctx.db().as_ref(), 1).await?;
        transaction.query_one_raw(Statement::from_string(DbBackend::Postgres, "SELECT 1")).await?;
        transaction.commit().await
      })
      .await
      .map_err(|_| "single-connection SELECT/commit deadline")?
      .map_err(|error| format!("concurrent database operation: {error}"))?;
      let edited = tokio::time::timeout(
        Duration::from_secs(30),
        execute_operation(
          &schema,
          ctx.db(),
          user(1),
          Request::new(format!(
            "mutation {{ credential3rdUpdate(data: {{ password: \"new-fixture-password\" }}, filter: {{ id: {{ eq: {stale} }} }}) {{ id }} }}",
          )),
        ),
      )
      .await
      .map_err(|_| "GraphQL credential-edit deadline")?;
      if !edited.errors.is_empty() {
        return Err(format!("credential edit: {:?}", edited.errors));
      }
      Ok(())
    }
    .await;
    // A failed phase closes the gate; only a committed edit permits a response.
    if let Some(release) = gate.take()
      && concurrent.is_ok()
    {
      let _ = release.send(());
    }
    concurrent?;
    let response = tokio::time::timeout(Duration::from_secs(60), checking.as_mut().unwrap())
      .await
      .map_err(|_| "credential-command response deadline")?
      .map_err(|error| format!("credential-command task: {error}"))?
      .map_err(|error| format!("credential-command HTTP: {error}"))?;
    checking.take();
    assert_eq!(response.status(), 409);
    assert_eq!(cookie(&fixture, stale).await, None, "old cookies must not overwrite an edited credential");
    assert_eq!(remote.calls.load(Ordering::SeqCst), 3);
    let password: String = fixture
      .bootstrap
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT password FROM credential3rd WHERE id = $1",
        [stale.into()],
      ))
      .await
      .unwrap()
      .unwrap()
      .try_get_by_index(0)
      .unwrap();
    assert_eq!(ctx.crypto().decrypt_string(&password).unwrap(), "new-fixture-password");
    Ok::<(), String>(())
  })
  .catch_unwind()
  .await;

  // Cleanup completes before surfacing coordination errors or primary panics.
  drop(gate.take());
  let mut cleanup_errors = Vec::new();
  if let Some(task) = checking.take() {
    task.abort();
    if let Err(error) = task.await
      && !error.is_cancelled()
    {
      cleanup_errors.push(format!("request task: {error}"));
    }
  }
  for result in [server.stop().await, remote_server.stop().await] {
    if let Err(error) = result {
      cleanup_errors.push(error);
    }
  }
  for result in [
    ctx.auth().runtime().close().await.map_err(|e| sea_orm::DbErr::Custom(e.to_string())),
    ctx.db().as_ref().clone().close().await,
  ] {
    if let Err(error) = result {
      cleanup_errors.push(error.to_string());
    }
  }
  let fixture_closed = AssertUnwindSafe(fixture.close()).catch_unwind().await;
  if fixture_closed.is_err() {
    cleanup_errors.push("PostgreSQL fixture close failed".into());
  }
  if !matches!(&outcome, Ok(Ok(()))) {
    eprintln!("Credential fixture cleanup completed: {} cleanup errors", cleanup_errors.len());
  }
  match outcome {
    Ok(result) => {
      assert!(cleanup_errors.is_empty(), "cleanup errors: {cleanup_errors:?}; test result: {result:?}");
      result.expect("credential command coordination failed");
    }
    Err(panic) => {
      if !cleanup_errors.is_empty() {
        eprintln!("Cleanup errors: {cleanup_errors:?}");
      }
      std::panic::resume_unwind(panic);
    }
  }
}
