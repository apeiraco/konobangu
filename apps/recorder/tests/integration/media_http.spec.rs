#![cfg(feature = "test-utils")]
#[path = "support/jxl.rs"]
mod jxl_support;
use std::{path::PathBuf, sync::Arc};

use axum::{Router, body::Body, http::Request};
use recorder::{app::AppContextTrait, media::MediaService, storage::StorageService, test_utils::app::TestingAppContext, web::controller::ControllerTrait};
use tower::ServiceExt;

struct Fixture {
  root: PathBuf,
  storage: StorageService,
}
impl Fixture {
  fn new() -> Self {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("../../temp/verification/fixtures/fixtures")
      .join(uuid::Uuid::now_v7().to_string());
    std::fs::create_dir_all(&root).unwrap();
    let storage = StorageService {
      data_dir: root.to_str().unwrap().into(),
      operator: StorageService::get_operator(root.to_str().unwrap()).unwrap(),
    };
    Self { root, storage }
  }
  async fn router(&self) -> Router {
    let ctx = Arc::new(
      TestingAppContext::builder()
        .storage(self.storage.clone())
        .media(MediaService::from_config(jxl_support::config()).await.unwrap())
        .build(),
    );
    recorder::web::controller::r#static::create(ctx.clone())
      .await
      .unwrap()
      .apply_to(Router::new())
      .with_state(ctx as Arc<dyn AppContextTrait>)
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(&self.root);
  }
}

#[tokio::test]
async fn review_range_boundaries() {
  let f = Fixture::new();
  f.storage.write("public/a.jpg", bytes::Bytes::from_static(b"0123456789")).await.unwrap();
  let router = f.router().await;
  let mut failures = Vec::new();
  for (range, expected_status, expected_length) in [
    ("bytes=99-100", 416, None),
    ("bytes=0-99", 206, Some("10")),
    ("bytes=-99", 206, Some("10")),
    ("bytes=9-2", 400, None),
    ("bytes=99-", 416, None),
    ("bytes=0-18446744073709551615", 206, Some("10")),
  ] {
    use futures::FutureExt;
    let result = std::panic::AssertUnwindSafe(
      router.clone().oneshot(
        Request::builder()
          .uri("/api/static/public/a.jpg?optimize=accept")
          .header("Range", range)
          .body(Body::empty())
          .unwrap(),
      ),
    )
    .catch_unwind()
    .await;
    match result {
      Ok(Ok(response)) => {
        let status = response.status().as_u16();
        let length = response.headers().get("content-length").and_then(|v| v.to_str().ok()).map(str::to_owned);
        let content_range = response.headers().get("content-range").and_then(|v| v.to_str().ok()).map(str::to_owned);
        let body = axum::body::to_bytes(response.into_body(), 1024)
          .await
          .map(|b| b.len())
          .map_err(|e| e.to_string());
        println!("{range}: status={status} length={length:?} content_range={content_range:?} body={body:?}");
        if status != expected_status || expected_length.is_some_and(|expected| length.as_deref() != Some(expected)) {
          failures.push(range);
        }
      }
      other => {
        println!("{range}: handler failed or panicked: {other:?}");
        failures.push(range);
      }
    }
  }
  assert!(failures.is_empty(), "Range contract failures: {failures:?}");
}

#[tokio::test]
async fn review_head_ignores_range() {
  let f = Fixture::new();
  f.storage.write("public/a.jpg", bytes::Bytes::from_static(b"0123456789")).await.unwrap();
  let response = f
    .router()
    .await
    .oneshot(
      Request::builder()
        .method("HEAD")
        .uri("/api/static/public/a.jpg?optimize=accept")
        .header("Range", "bytes=1-3")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  println!("HEAD Range: {} {:?}", response.status(), response.headers());
  assert_eq!(response.status(), 200);
  assert_eq!(response.headers()["content-length"], "10");
}

#[tokio::test]
async fn review_if_range_requires_exact_date() {
  let f = Fixture::new();
  f.storage.write("public/a.jpg", bytes::Bytes::from_static(b"0123456789")).await.unwrap();
  let response = f
    .router()
    .await
    .oneshot(
      Request::builder()
        .uri("/api/static/public/a.jpg?optimize=accept")
        .header("Range", "bytes=1-3")
        .header("If-Range", "Fri, 31 Dec 9999 23:59:59 GMT")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  println!("If-Range future date: {} {:?}", response.status(), response.headers());
  assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn review_legacy_collision_keeps_source_identity() {
  let f = Fixture::new();
  let service = MediaService::from_config(jxl_support::config()).await.unwrap();
  let mut raw = std::io::Cursor::new(Vec::new());
  image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(2, 2, image::Rgb([255, 0, 0])))
    .write_to(&mut raw, image::ImageFormat::Png)
    .unwrap();
  let red = bytes::Bytes::from(raw.into_inner());
  f.storage.write("public/a.png", red.clone()).await.unwrap();
  let mut raw = std::io::Cursor::new(Vec::new());
  image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(2, 2, image::Rgb([0, 0, 255])))
    .write_to(&mut raw, image::ImageFormat::Jpeg)
    .unwrap();
  f.storage.write("public/a.jpg", bytes::Bytes::from(raw.into_inner())).await.unwrap();
  let legacy = service.optimize_image_to_webp("a.png", red, None).await.unwrap();
  f.storage.write("public/a.webp", legacy.clone()).await.unwrap();
  let response = f
    .router()
    .await
    .oneshot(
      Request::builder()
        .uri("/api/static/public/a.jpg?optimize=accept")
        .header("Accept", "image/webp,image/jpeg")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  let mime = response.headers()["content-type"].clone();
  let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
  println!("Requested blue JPEG: mime={mime:?}, received red PNG legacy={}", body == legacy);
  assert_ne!(body, legacy, "Legacy fallback must not return another source's pixels");
  assert_eq!(mime, "image/jpeg");
  let blue = f.storage.read("public/a.jpg").await.unwrap().to_bytes();
  assert_eq!(body, blue);
  for ext in ["webp", "avif", "jxl"] {
    let path = format!("public/a.{ext}");
    f.storage.write(&path, legacy.clone()).await.unwrap();
    let response = f
      .router()
      .await
      .oneshot(Request::builder().uri(format!("/api/static/{path}")).body(Body::empty()).unwrap())
      .await
      .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap(), legacy);
  }
  f.storage.operator.write("public/a.jpg.derivatives.json", "broken").await.unwrap();
  let response = f
    .router()
    .await
    .oneshot(
      Request::builder()
        .uri("/api/static/public/a.jpg?optimize=accept")
        .header("Accept", "image/webp,image/jpeg")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap(), blue);
  let webp = service.optimize_image_to_webp("a.jpg", blue.clone(), None).await.unwrap();
  f.publish("public/a.jpg", &service, webp.clone()).await;
  let response = f
    .router()
    .await
    .oneshot(
      Request::builder()
        .uri("/api/static/public/a.jpg?optimize=accept")
        .header("Accept", "image/webp,image/jpeg")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap(), webp);
  let mut manifest = f.storage.load_derivatives("public/a.jpg").await.unwrap();
  manifest.entries[0].profile = "obsolete-profile".into();
  f.storage
    .operator
    .write("public/a.jpg.derivatives.json", serde_json::to_vec(&manifest).unwrap())
    .await
    .unwrap();
  let response = f
    .router()
    .await
    .oneshot(
      Request::builder()
        .uri("/api/static/public/a.jpg?optimize=accept")
        .header("Accept", "image/webp,image/jpeg")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap(), blue);
}

impl Fixture {
  async fn publish(&self, source: &str, media: &MediaService, data: bytes::Bytes) {
    use recorder::media::derivative::{self, DerivativeEntry, DerivativeManifest};
    let profile = media
      .derivative_plans()
      .into_iter()
      .find(|p| p.format == recorder::media::AutoOptimizeImageFormat::Webp)
      .unwrap()
      .profile;
    let source_sha256 = derivative::hash(&self.storage.read(source).await.unwrap().to_bytes());
    let sha256 = derivative::hash(&data);
    let path = derivative::derivative_path(source, &profile, &source_sha256, &sha256, recorder::media::AutoOptimizeImageFormat::Webp);
    self.storage.write(path.clone(), data.clone()).await.unwrap();
    let manifest = DerivativeManifest {
      version: derivative::VERSION,
      source_fingerprint: derivative::fingerprint(&self.storage.stat(source).await.unwrap()).unwrap(),
      source_sha256,
      entries: vec![DerivativeEntry {
        profile,
        format: recorder::media::AutoOptimizeImageFormat::Webp,
        path,
        length: data.len() as u64,
        sha256,
      }],
    };
    self
      .storage
      .operator
      .write(&derivative::manifest_path(source), serde_json::to_vec(&manifest).unwrap())
      .await
      .unwrap();
  }
}

#[tokio::test]
async fn review_http_body_lengths_multipart_limits_and_empty() {
  let f = Fixture::new();
  f.storage.write("public/ten.jpg", bytes::Bytes::from_static(b"0123456789")).await.unwrap();
  let media = MediaService::from_config(jxl_support::config()).await.unwrap();
  f.publish("public/ten.jpg", &media, bytes::Bytes::from_static(b"abcdefghij")).await;
  let router = f.router().await;
  for (accept, full) in [("image/jpeg", b"0123456789"), ("image/webp", b"abcdefghij")] {
    for (range, status, expected, content_range) in [
      ("bytes=99-100", 416, None, Some("bytes */10")),
      ("bytes=0-99", 206, Some(&full[..]), Some("bytes 0-9/10")),
      ("bytes=-99", 206, Some(&full[..]), Some("bytes 0-9/10")),
      ("bytes=-18446744073709551615", 206, Some(&full[..]), Some("bytes 0-9/10")),
      ("bytes=9-2", 400, None, None),
      ("bytes=99-", 416, None, Some("bytes */10")),
      ("bytes=0-18446744073709551615", 206, Some(&full[..]), Some("bytes 0-9/10")),
      ("bytes=-0", 416, None, Some("bytes */10")),
      ("bytes=0-1,-0", 206, Some(&full[..2]), Some("bytes 0-1/10")),
      ("bytes=-0,0-1", 206, Some(&full[..2]), Some("bytes 0-1/10")),
      ("bytes=99-100,-0,0-1", 206, Some(&full[..2]), Some("bytes 0-1/10")),
      ("bytes=00-01", 206, Some(&full[..2]), Some("bytes 0-1/10")),
      ("bytes=99-2", 416, None, Some("bytes */10")),
      ("bytes=99-100,1-3", 206, Some(&full[1..4]), Some("bytes 1-3/10")),
      ("bytes=0-99999999999999999999999999999999999", 400, None, None),
      ("widgets=1-3", 200, Some(&full[..]), None),
      ("bytes=0-4,2-8", 200, Some(&full[..]), None),
    ] {
      let response = router
        .clone()
        .oneshot(
          Request::builder()
            .uri("/api/static/public/ten.jpg?optimize=accept")
            .header("Accept", accept)
            .header("Range", range)
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
      assert_eq!(response.status(), status, "{accept}: {range}");
      let headers = response.headers().clone();
      assert_eq!(headers.get("content-range").map(|h| h.to_str().unwrap()), content_range);
      assert!(headers.get_all("vary").iter().any(|v| v == "Accept"));
      let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
      if let Some(expected) = expected {
        assert_eq!(&body[..], expected);
        assert_eq!(headers["content-length"].to_str().unwrap().parse::<usize>().unwrap(), body.len());
      }
    }
    let too_many = (0..17).map(|_| "0-0").collect::<Vec<_>>().join(",");
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .uri("/api/static/public/ten.jpg?optimize=accept")
          .header("Accept", accept)
          .header("Range", format!("bytes={too_many}"))
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(&axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap()[..], full);
    for (range, first, last) in [("bytes=7-9,0-2", 7..10, 0..3), ("bytes=0-1,  4-5", 0..2, 4..6)] {
      let response = router
        .clone()
        .oneshot(
          Request::builder()
            .uri("/api/static/public/ten.jpg?optimize=accept")
            .header("Accept", accept)
            .header("Range", range)
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
      assert_eq!(response.status(), 206);
      let headers = response.headers().clone();
      let boundary = headers["content-type"].to_str().unwrap().split("boundary=").nth(1).unwrap();
      let mime = if accept == "image/webp" { "image/webp" } else { "image/jpeg" };
      let expected = format!(
        "--{boundary}\r\nContent-Type: {mime}\r\nContent-Range: bytes {}-{}/10\r\n\r\n{}\r\n--{boundary}\r\nContent-Type: {mime}\r\nContent-Range: bytes \
         {}-{}/10\r\n\r\n{}\r\n--{boundary}--\r\n",
        first.start,
        first.end - 1,
        std::str::from_utf8(&full[first.clone()]).unwrap(),
        last.start,
        last.end - 1,
        std::str::from_utf8(&full[last.clone()]).unwrap()
      );
      let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
      assert_eq!(&body[..], expected.as_bytes());
      assert_eq!(headers["content-length"].to_str().unwrap().parse::<usize>().unwrap(), body.len());
    }
  }
  f.storage.write("public/empty.jpg", bytes::Bytes::new()).await.unwrap();
  let response = router
    .oneshot(
      Request::builder()
        .uri("/api/static/public/empty.jpg?optimize=accept")
        .header("Range", "bytes=999-")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), 200);
  assert_eq!(response.headers()["content-length"], "0");
  assert!(axum::body::to_bytes(response.into_body(), 100).await.unwrap().is_empty());
}

#[tokio::test]
async fn review_range_member_limit_and_duplicate_headers() {
  let f = Fixture::new();
  let full = bytes::Bytes::from_static(b"0123456789abcdefghijklmnopqrstuv");
  f.storage.write("public/limit.jpg", full.clone()).await.unwrap();
  let media = MediaService::from_config(jxl_support::config()).await.unwrap();
  f.publish("public/limit.jpg", &media, full.clone()).await;
  let router = f.router().await;
  let members = (0..16).map(|i| format!("{0}-{0}", i * 2)).collect::<Vec<_>>().join(",");
  let oversized_invalid = format!("bytes={}not-an-integer", "-0,".repeat(16));
  for accept in ["image/jpeg", "image/webp"] {
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .uri("/api/static/public/limit.jpg?optimize=accept")
          .header("Accept", accept)
          .header("Range", format!("bytes={members}"))
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), 206);
    let headers = response.headers().clone();
    assert!(!headers.contains_key("content-range"));
    let boundary = headers["content-type"].to_str().unwrap().split("boundary=").nth(1).unwrap();
    let mut expected = String::new();
    for i in (0..32).step_by(2) {
      expected.push_str(&format!(
        "--{boundary}\r\nContent-Type: {accept}\r\nContent-Range: bytes {i}-{i}/32\r\n\r\n{}\r\n",
        full[i] as char
      ));
    }
    expected.push_str(&format!("--{boundary}--\r\n"));
    let body = axum::body::to_bytes(response.into_body(), 65536).await.unwrap();
    assert_eq!(&body[..], expected.as_bytes());
    assert_eq!(headers["content-length"].to_str().unwrap().parse::<usize>().unwrap(), body.len());
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .uri("/api/static/public/limit.jpg?optimize=accept")
          .header("Accept", accept)
          .header("Range", &oversized_invalid)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), 200, "Member count must be checked before parsing");
    assert_eq!(response.headers()["content-length"], "32");
    assert!(!response.headers().contains_key("content-range"));
    assert_eq!(axum::body::to_bytes(response.into_body(), 65536).await.unwrap(), full);
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .uri("/api/static/public/limit.jpg?optimize=accept")
          .header("Accept", accept)
          .header("Range", "bytes=0-1")
          .header("Range", "bytes=4-5")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), 400);
    let _ = axum::body::to_bytes(response.into_body(), 65536).await.unwrap();
  }
}

#[tokio::test]
async fn review_head_and_if_range_only_current_strong_etag() {
  let f = Fixture::new();
  let media = MediaService::from_config(jxl_support::config()).await.unwrap();
  f.storage.write("public/ten.jpg", bytes::Bytes::from_static(b"0123456789")).await.unwrap();
  f.publish("public/ten.jpg", &media, bytes::Bytes::from_static(b"abcdefghij")).await;
  let router = f.router().await;
  for accept in ["image/jpeg", "image/webp"] {
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .uri("/api/static/public/ten.jpg?optimize=accept")
          .header("Accept", accept)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    let etag = response.headers()["etag"].clone();
    let date = response.headers()["last-modified"].clone();
    let strong = accept == "image/webp";
    for (condition, partial) in [
      (etag.to_str().unwrap(), strong),
      ("\"other-variant\"", false),
      ("W/\"weak\"", false),
      (date.to_str().unwrap(), false),
      ("Fri, 31 Dec 9999 23:59:59 GMT", false),
      ("Sun, 06 Nov 1994 08:49:37 GMT", false),
      ("invalid-date", false),
    ] {
      let response = router
        .clone()
        .oneshot(
          Request::builder()
            .uri("/api/static/public/ten.jpg?optimize=accept")
            .header("Accept", accept)
            .header("Range", "bytes=1-3")
            .header("If-Range", condition)
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
      assert_eq!(response.status(), if partial { 206 } else { 200 });
      assert_eq!(
        axum::body::to_bytes(response.into_body(), 100).await.unwrap().len(),
        if partial { 3 } else { 10 }
      );
    }
    for range in ["bytes=1-3", "bytes=99-", "bytes=broken"] {
      let response = router
        .clone()
        .oneshot(
          Request::builder()
            .method("HEAD")
            .uri("/api/static/public/ten.jpg?optimize=accept")
            .header("Accept", accept)
            .header("Range", range)
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
      assert_eq!(response.status(), 200);
      assert_eq!(response.headers()["content-length"], "10");
      assert!(!response.headers().contains_key("content-range"));
      assert!(axum::body::to_bytes(response.into_body(), 100).await.unwrap().is_empty());
    }
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .method("HEAD")
          .uri("/api/static/public/ten.jpg?optimize=accept")
          .header("Accept", accept)
          .header("Range", "bytes=broken")
          .header("If-None-Match", etag)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), 304);
    assert!(!response.headers().contains_key("content-range"));
    assert!(axum::body::to_bytes(response.into_body(), 100).await.unwrap().is_empty());
  }
}

#[cfg(feature = "jxl")]
#[tokio::test]
async fn review_verified_larger_ordinary_jxl_remains_preferred() {
  use recorder::media::{
    AutoOptimizeImageFormat as F,
    derivative::{self, DerivativeEntry, DerivativeManifest},
  };
  let f = Fixture::new();
  // A small fixed cover tests negotiation, while the separate release corpus
  // covers production-sized quality and resource limits.
  let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(64, 64, |x, y| {
    let v = if x % 31 < 2 || y % 29 < 2 { 0 } else { 255 };
    image::Rgb([v, v, v])
  }));
  let mut png = std::io::Cursor::new(Vec::new());
  image.write_to(&mut png, image::ImageFormat::Png).unwrap();
  let input: bytes::Bytes = png.into_inner().into();
  let config = jxl_support::config();
  assert_eq!(config.auto_optimize_formats, [F::Jxl, F::Webp]);
  let service = MediaService::from_config(config).await.unwrap();
  f.storage.write("public/cover.png", input.clone()).await.unwrap();
  let webp = service.optimize_cover_to_webp(input.clone(), None).await.unwrap();
  let jxl = service.optimize_cover_to_jxl(input.clone(), None).await.unwrap();
  let normalized = recorder::media::normalize_cover(recorder::media::decode(&input, &service.config).unwrap(), &service.config).unwrap();
  let decoded = jxl_support::decode(&jxl);
  assert_eq!(decoded.dimensions(), (normalized.width(), normalized.height()));
  assert!(decoded.pixels().all(|p| p[3] == 255));
  assert!(
    jxl.len() > webp.len(),
    "The fixed regression must exercise a larger JXL: {} versus {}",
    jxl.len(),
    webp.len()
  );
  f.publish("public/cover.png", &service, webp.clone()).await;
  let mut manifest: DerivativeManifest = f.storage.load_derivatives("public/cover.png").await.unwrap();
  let plan = service.derivative_plans().into_iter().find(|p| p.format == F::Jxl).unwrap();
  let sha256 = derivative::hash(&jxl);
  let path = derivative::derivative_path("public/cover.png", &plan.profile, &manifest.source_sha256, &sha256, F::Jxl);
  f.storage.write(path.clone(), jxl.clone()).await.unwrap();
  manifest.entries.push(DerivativeEntry {
    profile: plan.profile,
    format: F::Jxl,
    path,
    length: jxl.len() as u64,
    sha256: sha256.clone(),
  });
  f.storage
    .operator
    .write("public/cover.png.derivatives.json", serde_json::to_vec(&manifest).unwrap())
    .await
    .unwrap();
  let ctx = Arc::new(TestingAppContext::builder().storage(f.storage.clone()).media(service).build());
  let router = recorder::web::controller::r#static::create(ctx.clone())
    .await
    .unwrap()
    .apply_to(Router::new())
    .with_state(ctx as Arc<dyn AppContextTrait>);
  for (accept, mime, expected) in [
    ("image/webp,image/jxl", "image/jxl", jxl.clone()),
    ("image/jxl,image/webp", "image/jxl", jxl.clone()),
    ("image/jxl;q=0,image/webp", "image/webp", webp.clone()),
    ("image/*", "image/webp", webp),
  ] {
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .uri("/api/static/public/cover.png?optimize=accept")
          .header("Accept", accept)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], mime);
    if mime == "image/jxl" {
      assert_eq!(response.headers()["etag"], format!("\"{sha256}\""));
    }
    assert_eq!(axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap(), expected);
  }
}

#[tokio::test]
async fn review_if_range_same_second_variants_and_source_replacement() {
  let f = Fixture::new();
  let media = MediaService::from_config(jxl_support::config()).await.unwrap();
  let source = "public/same-second.jpg";
  let fixed = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
  let set_time = |path: &str| {
    std::fs::File::options()
      .write(true)
      .open(f.root.join(path))
      .unwrap()
      .set_times(std::fs::FileTimes::new().set_modified(fixed))
      .unwrap();
  };
  f.storage.write(source, bytes::Bytes::from_static(b"0123456789")).await.unwrap();
  set_time(source);
  f.publish(source, &media, bytes::Bytes::from_static(b"abcdefghij")).await;
  let manifest = f.storage.load_derivatives(source).await.unwrap();
  set_time(&manifest.entries[0].path);
  let router = f.router().await;
  let get = |accept: &str, condition: Option<&str>| {
    let mut request = Request::builder()
      .uri("/api/static/public/same-second.jpg?optimize=accept")
      .header("Accept", accept);
    if let Some(condition) = condition {
      request = request.header("Range", "bytes=1-3").header("If-Range", condition);
    }
    request.body(Body::empty()).unwrap()
  };
  let original = router.clone().oneshot(get("image/jpeg", None)).await.unwrap();
  let derivative = router.clone().oneshot(get("image/webp", None)).await.unwrap();
  let date = original.headers()["last-modified"].to_str().unwrap().to_owned();
  assert_eq!(derivative.headers()["last-modified"], date);
  let derivative_etag = derivative.headers()["etag"].to_str().unwrap().to_owned();
  for accept in ["image/jpeg", "image/webp"] {
    let response = router.clone().oneshot(get(accept, Some(&date))).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(axum::body::to_bytes(response.into_body(), 100).await.unwrap().len(), 10);
  }
  let response = router.clone().oneshot(get("image/jpeg", Some(&derivative_etag))).await.unwrap();
  assert_eq!(response.status(), 200);
  f.storage.write(source, bytes::Bytes::from_static(b"9876543210")).await.unwrap();
  set_time(source);
  let response = router.oneshot(get("image/jpeg", Some(&date))).await.unwrap();
  assert_eq!(response.status(), 200);
  assert_eq!(response.headers()["last-modified"], date);
  assert_eq!(&axum::body::to_bytes(response.into_body(), 100).await.unwrap()[..], b"9876543210");
}
