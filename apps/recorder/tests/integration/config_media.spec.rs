//! Configuration tests spawn an isolated process rather than mutate global env.
#[path = "support/jxl.rs"]
mod jxl_support;
use std::{io::Cursor, path::PathBuf, process::Command};

use bytes::Bytes;
use image::{DynamicImage, GenericImageView, ImageFormat, RgbImage, RgbaImage};
use recorder::{
  app::{AppConfig, Environment},
  media::{AutoOptimizeImageFormat, EncodeWebpOptions, MediaService},
};

#[tokio::test]
async fn config_environment_child() {
  if std::env::var("CONFIG_TEST_CHILD").as_deref() != Ok("1") {
    return;
  }
  let path = std::env::var("CONFIG_TEST_PATH").unwrap();
  let dotenv = std::env::var("CONFIG_TEST_DOTENV").unwrap();
  AppConfig::load_dotenv(&Environment::Testing, Some(&dotenv)).await.unwrap();
  let result = AppConfig::load_config(&Environment::Testing, Some(&path)).await;
  if let Ok(expected) = std::env::var("CONFIG_TEST_ERROR") {
    let error = result.unwrap_err().to_string();
    assert!(error.contains(&expected), "expected a configuration diagnostic");
    assert!(!error.contains("SECRET_FIXTURE"), "configuration values must be redacted");
  } else {
    let config = result.unwrap();
    assert_eq!(config.server.host, std::env::var("CONFIG_TEST_HOST").unwrap());
    assert_eq!(config.server.port, 6007);
    let auto_migrate = std::env::var("CONFIG_TEST_AUTO_MIGRATE").is_ok_and(|value| value == "true");
    assert_eq!(config.database.auto_migrate, auto_migrate);
    assert!(config.database.uri.ends_with(&std::env::var("CONFIG_TEST_DATABASE").unwrap()));
    let expected: Vec<AutoOptimizeImageFormat> =
      serde_json::from_str(&std::env::var("CONFIG_TEST_FORMATS").unwrap_or_else(|_| "[\"image/webp\"]".into())).unwrap();
    assert_eq!(config.media.auto_optimize_formats, expected);
  }
}

#[test]
fn native_configuration_formats_types_dotenv_and_errors() {
  let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../../temp/verification/fixtures/config")
    .join(uuid::Uuid::now_v7().to_string());
  std::fs::create_dir_all(&root).unwrap();
  let dotenv = root.join("fixture.env");
  std::fs::write(
    &dotenv,
    "SERVER__HOST=dotenv-host\nDATABASE__URL=postgres://fixture/dotenv-database\nSERVER__PORT=6007\n",
  )
  .unwrap();
  let native = serde_json::json!({"server":{"host":"file-host","port":5001},"database":{"url":"postgres://fixture/file-database","migration":{"auto_run":false}},"auth":{"provider":{"type":"basic","username":"fixture","password":"fixture"},"session":{"database_url":"postgres://fixture/identity","public_url":"http://127.0.0.1:5001"}},"logger":{"enable":false,"level":"off","format":"compact"},"media":{"auto_optimize_formats":["image/webp"]}});
  let base = || {
    let mut c = Command::new(std::env::current_exe().unwrap());
    c.env_clear()
      .arg("--exact")
      .arg("config_environment_child")
      .arg("--nocapture")
      .env("CONFIG_TEST_CHILD", "1")
      .env("CONFIG_TEST_DOTENV", &dotenv)
      .env("CONFIG_TEST_HOST", "process-host")
      .env("CONFIG_TEST_DATABASE", "canonical-database")
      .env("SERVER__HOST", "process-host")
      .env("DATABASE__URL", "postgres://fixture/canonical-database");
    c
  };
  // Native YAML/JSON/TOML all flow through the real Figment providers.
  for extension in ["json", "toml", "yaml"] {
    let path = root.join(format!("config.{extension}"));
    let body=match extension { "json"=>serde_json::to_string(&native).unwrap(), "yaml"=>"server:\n  host: file-host\n  port: 5001\ndatabase:\n  url: postgres://fixture/file-database\n  migration:\n    auto_run: false\nauth:\n  provider:\n    type: basic\n    username: fixture\n    password: fixture\n  session:\n    database_url: postgres://fixture/identity\n    public_url: http://127.0.0.1:5001\nlogger:\n  enable: false\n  level: off\n  format: compact\nmedia:\n  auto_optimize_formats: [image/webp]\n".into(), _=>"[server]\nhost='file-host'\nport=5001\n[database]\nurl='postgres://fixture/file-database'\n[database.migration]\nauto_run=false\n[auth.provider]\ntype='basic'\nusername='fixture'\npassword='fixture'\n[auth.session]\ndatabase_url='postgres://fixture/identity'\npublic_url='http://127.0.0.1:5001'\n[logger]\nenable=false\nlevel='off'\nformat='compact'\n[media]\nauto_optimize_formats=['image/webp']\n".into() };
    std::fs::write(&path, body).unwrap();
    let output = base().env("CONFIG_TEST_PATH", &path).output().unwrap();
    assert!(
      output.status.success(),
      "native {extension}: {}{}",
      String::from_utf8_lossy(&output.stdout),
      String::from_utf8_lossy(&output.stderr)
    );
    let output = base()
      .env("CONFIG_TEST_PATH", &path)
      .env("SERVER__HOST", "canonical-host")
      .env("CONFIG_TEST_HOST", "canonical-host")
      .output()
      .unwrap();
    assert!(output.status.success(), "canonical host wins");
  }
  let path = root.join("config.json");
  let output = base()
    .env("CONFIG_TEST_PATH", &path)
    .env_remove("SERVER__HOST")
    .env_remove("DATABASE__URL")
    .env("CONFIG_TEST_HOST", "dotenv-host")
    .env("CONFIG_TEST_DATABASE", "dotenv-database")
    .output()
    .unwrap();
  assert!(output.status.success(), "canonical dotenv values beat files");
  let defaults = if cfg!(feature = "jxl") {
    vec![AutoOptimizeImageFormat::Jxl, AutoOptimizeImageFormat::Webp]
  } else {
    vec![AutoOptimizeImageFormat::Webp]
  };
  assert_eq!(recorder::media::MediaConfig::default().auto_optimize_formats, defaults);
  let policy_path = root.join("media-policy.json");
  for (file_formats, canonical, expected) in [
    (None, None, defaults.clone()),
    (Some(vec![AutoOptimizeImageFormat::Webp]), None, vec![AutoOptimizeImageFormat::Webp]),
    (Some(vec![]), None, vec![]),
    (Some(vec![AutoOptimizeImageFormat::Webp]), Some("[]"), vec![]),
    (Some(vec![]), Some("[\"image/webp\"]"), vec![AutoOptimizeImageFormat::Webp]),
  ] {
    let mut body = native.clone();
    body["media"] = match file_formats {
      Some(formats) => serde_json::json!({"auto_optimize_formats": formats}),
      None => serde_json::json!({}),
    };
    std::fs::write(&policy_path, serde_json::to_vec(&body).unwrap()).unwrap();
    let mut command = base();
    if let Some(value) = canonical {
      command.env("MEDIA__AUTO_OPTIMIZE_FORMATS", value);
    }
    let output = command
      .env("CONFIG_TEST_PATH", &policy_path)
      .env("CONFIG_TEST_FORMATS", serde_json::to_string(&expected).unwrap())
      .output()
      .unwrap();
    assert!(
      output.status.success(),
      "compiled defaults and explicit media policies: {}",
      String::from_utf8_lossy(&output.stdout)
    );
  }
  // The shipped file and empty mixin must inherit the same feature-aware
  // default.
  let output = base()
    .env("CONFIG_TEST_PATH", PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("recorder.config.toml"))
    .env("AUTH__PROVIDER__TYPE", "basic")
    .env("AUTH__PROVIDER__USERNAME", "fixture")
    .env("AUTH__PROVIDER__PASSWORD", "fixture")
    .env("AUTH__SESSION__DATABASE_URL", "postgres://fixture/identity")
    .env("AUTH__SESSION__PUBLIC_URL", "http://127.0.0.1:6007")
    .env("CONFIG_TEST_FORMATS", serde_json::to_string(&defaults).unwrap())
    .env("CONFIG_TEST_AUTO_MIGRATE", "true")
    .output()
    .unwrap();
  assert!(
    output.status.success(),
    "shipped configuration defaults: {}",
    String::from_utf8_lossy(&output.stdout)
  );
  let bad = root.join("invalid.toml");
  for (body, diagnostic) in [
    ("[server]\nhost='{{ SECRET_FIXTURE }}'", "Invalid configuration"),
    ("[server]\nhost='SECRET_FIXTURE'\nport='SECRET_FIXTURE'", "Invalid configuration"),
    ("broken=SECRET_FIXTURE", "Invalid configuration"),
  ] {
    std::fs::write(&bad, body).unwrap();
    let output = base()
      .env("CONFIG_TEST_PATH", &bad)
      .env("CONFIG_TEST_ERROR", diagnostic)
      .env_remove("SERVER__PORT")
      .env("SERVER__PORT", "SECRET_FIXTURE")
      .output()
      .unwrap();
    assert!(
      output.status.success(),
      "invalid values produce redacted diagnostics: {}{}",
      String::from_utf8_lossy(&output.stdout),
      String::from_utf8_lossy(&output.stderr)
    );
  }
  let output = base()
    .env("CONFIG_TEST_PATH", root.join("missing.toml"))
    .env("CONFIG_TEST_ERROR", "does not exist")
    .output()
    .unwrap();
  assert!(output.status.success());
  std::fs::remove_dir_all(root).unwrap();
}

fn png(alpha: bool) -> Bytes {
  let image = if alpha {
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(7, 5, image::Rgba([80, 160, 240, 64])))
  } else {
    DynamicImage::ImageRgb8(RgbImage::from_pixel(7, 5, image::Rgb([80, 160, 240])))
  };
  let mut bytes = Cursor::new(Vec::new());
  image.write_to(&mut bytes, ImageFormat::Png).unwrap();
  Bytes::from(bytes.into_inner())
}

#[cfg(not(feature = "jxl"))]
#[tokio::test]
async fn webp_only_worker_rejects_automatic_jxl() {
  let config = serde_json::from_value::<recorder::media::MediaConfig>(serde_json::json!({
    "auto_optimize_formats": ["image/jxl", "image/webp"]
  }))
  .unwrap();
  let Err(error) = MediaService::from_config(config).await else {
    panic!("Automatic JXL configuration must be rejected without the feature");
  };
  assert!(error.to_string().contains("jxl feature is not enabled"));
}

#[tokio::test]
async fn media_webp_decodes_dimensions_alpha_and_validates_quality() {
  let config = jxl_support::config();
  assert_eq!(config.auto_optimize_formats, recorder::media::MediaConfig::default().auto_optimize_formats);
  let service = MediaService::from_config(config).await.unwrap();
  for alpha in [false, true] {
    for quality in [0.0, 100.0] {
      let bytes = service
        .optimize_image_to_webp("fixture.png", png(alpha), Some(EncodeWebpOptions { quality: Some(quality) }))
        .await
        .unwrap();
      let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::WebP).unwrap();
      assert_eq!(decoded.dimensions(), (7, 5));
      if alpha {
        assert_eq!(decoded.into_rgba8().get_pixel(0, 0).0[3], 64);
      }
    }
  }
  assert!(service.optimize_image_to_webp("broken.png", Bytes::from_static(b"broken"), None).await.is_err());
  for quality in [-1.0, 101.0, f32::NAN, f32::INFINITY] {
    assert!(
      service
        .optimize_image_to_webp("fixture.png", png(true), Some(EncodeWebpOptions { quality: Some(quality) }))
        .await
        .is_err()
    );
  }
  #[cfg(not(feature = "jxl"))]
  assert!(
    service
      .optimize_image_to_jxl("fixture.png", png(true), None)
      .await
      .unwrap_err()
      .to_string()
      .contains("jxl feature is not enabled")
  );
  assert_eq!(mime_guess::from_path("fixture.webp").first().unwrap().as_ref(), "image/webp");
  assert_eq!(mime_guess::from_path("fixture.jxl").first().unwrap().as_ref(), "image/jxl");
}
#[tokio::test]
async fn media_existing_jxl_is_served_without_encoder() {
  let service = recorder::storage::StorageService {
    data_dir: "memory".into(),
    operator: opendal::Operator::new(opendal::services::Memory::default()).unwrap(),
  };
  let original = Bytes::from_static(b"legacy-jxl-fixture");
  service.write("public/existing.jxl", original.clone()).await.unwrap();
  let response = service.serve_file("public/existing.jxl", None).await.unwrap();
  assert_eq!(response.headers()[http::header::CONTENT_TYPE], "image/jxl");
  let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
  assert_eq!(body, original);
}

#[cfg(feature = "jxl")]
#[tokio::test]
async fn media_optional_jxl_preset_rgb_and_explicit_rgba_rejection() {
  let service = MediaService::from_config(jxl_support::config()).await.unwrap();
  let bytes = service.optimize_image_to_jxl("fixture.png", png(false), None).await.unwrap();
  assert_eq!(jxl_support::decode(&bytes).dimensions(), (7, 5));
  let error = service.optimize_image_to_jxl("fixture.png", png(true), None).await.unwrap_err();
  assert!(error.to_string().contains("RGBA"));
  let bytes = service.optimize_cover_to_jxl(png(true), None).await.unwrap();
  assert!(jxl_support::decode(&bytes).pixels().all(|p| p[3] == 255));
  assert!(service.optimize_image_to_jxl("broken.png", Bytes::from_static(b"broken"), None).await.is_err());
  assert!(
    service
      .optimize_cover_to_jxl(png(false), Some(recorder::media::EncodeJxlOptions { preset_version: 2 }))
      .await
      .is_err()
  );
  for field in ["distance", "effort", "quality", "speed", "progressive"] {
    assert!(serde_json::from_value::<recorder::media::EncodeJxlOptions>(serde_json::json!({"preset_version":1,field:0})).is_err());
  }
}

#[test]
fn cron_dst_repeated_occurrences_and_nonexistent_local_time() {
  use chrono::{TimeZone, Utc};
  use recorder::models::cron::Model;
  let before = Utc.with_ymd_and_hms(2030, 11, 3, 4, 0, 0).unwrap();
  let first = Model::calculate_next_run_after("0 30 1 * * *", "America/New_York", before).unwrap();
  let second = Model::calculate_next_run_after("0 30 1 * * *", "America/New_York", first).unwrap();
  assert_eq!(first, Utc.with_ymd_and_hms(2030, 11, 3, 5, 30, 0).unwrap());
  assert_eq!(second, Utc.with_ymd_and_hms(2030, 11, 3, 6, 30, 0).unwrap());
  let before = Utc.with_ymd_and_hms(2030, 3, 10, 5, 0, 0).unwrap();
  assert_eq!(
    Model::calculate_next_run_after("0 30 2 * * *", "America/New_York", before).unwrap(),
    Utc.with_ymd_and_hms(2030, 3, 11, 6, 30, 0).unwrap()
  );
}
