use std::sync::Arc;

use recorder::{
  auth::{
    config::AuthSessionConfig,
    session::{
      pending::{PendingStoreConfig, PostgresPendingStore},
      runtime::{GRANT_KEY, SessionRuntime},
    },
  },
  models::auth::Model,
};
use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};
use securitydept_oidc_client::PendingOauthStore;
use tower_sessions::{
  Session, SessionStore,
  session::{Id, Record},
};

use super::support::RlsFixture;

#[tokio::test]
async fn persistent_identity_pending_and_revocation_survive_late_saves() {
  let fixture = RlsFixture::new().await;
  let config: AuthSessionConfig = serde_json::from_value(
    serde_json::json!({ "identity_database_uri": fixture.identity_uri, "external_base_url": "http://127.0.0.1:5001", "session_cookie_secure": false }),
  )
  .unwrap();
  let runtime = SessionRuntime::new(config.clone()).await.unwrap();
  assert_eq!(runtime.identity_db.get_postgres_connection_pool().options().get_max_connections(), 3);
  assert_eq!(runtime.session_pool.options().get_max_connections(), 2);
  let restarted = SessionRuntime::new(config.clone()).await.unwrap();
  let mut invalid_session = config.clone();
  invalid_session.external_base_url = "https://app.example/".parse().unwrap();
  assert!(
    SessionRuntime::new(invalid_session).await.is_err(),
    "insecure non-loopback cookies fail configuration"
  );
  let mut invalid_session = config.clone();
  invalid_session.session_idle_seconds = 0;
  assert!(SessionRuntime::new(invalid_session).await.is_err());
  let mut oidc: recorder::auth::config::OidcAuthConfig = serde_json::from_value(serde_json::json!({ "oidc_issuer": "https://not-discovered.invalid", "oidc_audience": "legacy-api", "oidc_client_id": "konobangu", "oidc_client_secret": "fixture-only", "identity_database_uri": config.identity_database_uri, "external_base_url": config.external_base_url, "session_cookie_secure": false })).unwrap();
  assert!(
    recorder::auth::oidc::OidcAuthService::new(oidc.clone(), SessionRuntime::new(config.clone()).await.unwrap())
      .await
      .is_err()
  );
  oidc.audience = oidc.client_id.clone();
  oidc.extra_claims = Some(std::collections::HashMap::from([("restricted".into(), Some("required".into()))]));
  assert!(
    recorder::auth::oidc::OidcAuthService::new(oidc, SessionRuntime::new(config.clone()).await.unwrap())
      .await
      .is_err()
  );
  assert!(
    serde_json::from_value::<recorder::auth::AuthConfig>(serde_json::json!({"auth_type": "oidc"})).is_err(),
    "missing OIDC config cannot become development identity"
  );
  let db = &runtime.identity_db;
  let (a, a_again) = tokio::join!(
    Model::find_or_create_oidc(db, "https://issuer-a.example", "same-subject"),
    Model::find_or_create_oidc(db, "https://issuer-a.example", "same-subject")
  );
  let a = a.unwrap();
  assert_eq!(a.id, a_again.unwrap().id);
  let b = Model::find_or_create_oidc(db, "https://issuer-b.example", "same-subject").await.unwrap();
  assert_ne!(a.subscriber_id, b.subscriber_id);
  assert_ne!(a.subscriber_id, 1, "new OIDC identities cannot claim the Basic seed");
  let pending = PostgresPendingStore::from_config(&PendingStoreConfig {
    pool: Some(db.get_postgres_connection_pool().clone()),
    ttl_seconds: 300,
  });
  pending
    .insert("state-once".into(), "nonce".into(), Some("verifier".into()), None)
    .await
    .unwrap();
  let (first, second) = tokio::join!(pending.take("state-once"), pending.take("state-once"));
  assert_eq!(usize::from(first.unwrap().is_some()) + usize::from(second.unwrap().is_some()), 1);
  let expired = PostgresPendingStore::from_config(&PendingStoreConfig {
    pool: Some(db.get_postgres_connection_pool().clone()),
    ttl_seconds: -1,
  });
  expired.insert("expired".into(), "nonce".into(), None, None).await.unwrap();
  assert!(pending.take("expired").await.unwrap().is_none());
  let store = Arc::new(runtime.store.clone());
  let session = Session::new(None, store.clone(), None);
  let grant = runtime.create_grant(a.id).await.unwrap();
  session.insert(GRANT_KEY, grant).await.unwrap();
  session.save().await.unwrap();
  let old_id = session.id().unwrap();
  let late_record = store.load(&old_id).await.unwrap().unwrap();
  let restored = Session::new(Some(old_id), Arc::new(restarted.store.clone()), None);
  assert_eq!(restarted.authenticate(&restored).await.unwrap().id, a.id);
  session.cycle_id().await.unwrap();
  session.save().await.unwrap();
  assert_ne!(session.id().unwrap(), old_id);
  assert!(runtime.authenticate(&Session::new(Some(old_id), store.clone(), None)).await.is_err());
  runtime.revoke(&session).await.unwrap();
  // Even an in-flight store save recreating the old ID cannot undo the durable
  // grant revocation.
  store.save(&late_record).await.unwrap();
  assert!(runtime.authenticate(&Session::new(Some(old_id), store.clone(), None)).await.is_err());
  runtime.cleanup().await.unwrap();
  assert!(runtime.authenticate(&restored).await.is_err());
  let mut record = Record {
    id: Id::default(),
    data: Default::default(),
    expiry_date: tower_sessions::cookie::time::OffsetDateTime::now_utc() - tower_sessions::cookie::time::Duration::seconds(1),
  };
  store.create(&mut record).await.unwrap();
  assert!(store.load(&record.id).await.unwrap().is_none());
  let mut colliding = late_record.clone();
  store.create(&mut colliding).await.unwrap();
  assert_ne!(colliding.id, late_record.id, "adapter must write back the mature store's regenerated ID");
  let absolute = Session::new(None, store.clone(), None);
  let grant = runtime.create_grant(b.id).await.unwrap();
  absolute.insert(GRANT_KEY, grant).await.unwrap();
  absolute.save().await.unwrap();
  sqlx::query(
    "UPDATE auth_session.login_grant SET created_at = CURRENT_TIMESTAMP - INTERVAL '2 seconds', expires_at = CURRENT_TIMESTAMP - INTERVAL '1 second' WHERE id \
     = $1",
  )
  .bind(grant)
  .execute(db.get_postgres_connection_pool())
  .await
  .unwrap();
  assert!(
    runtime.authenticate(&absolute).await.is_err(),
    "an active cookie cannot extend the absolute login deadline"
  );
  // The identity role cannot read any private business table, even with a
  // forged setting.
  assert!(
    db.query_all_raw(Statement::from_string(DbBackend::Postgres, "SELECT * FROM subscriptions"))
      .await
      .is_err()
  );
  let business = Database::connect(&fixture.app_uri).await.unwrap();
  assert!(
    business
      .query_all_raw(Statement::from_string(DbBackend::Postgres, "SELECT * FROM auth_identity.auth"))
      .await
      .is_err()
  );
  business.close().await.unwrap();
  runtime.close().await.unwrap();
  assert!(runtime.session_pool.is_closed());
  assert!(runtime.identity_db.get_postgres_connection_pool().is_closed());
  restarted.close().await.unwrap();
  fixture.close().await;
}
