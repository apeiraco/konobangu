#[path = "support/jxl.rs"]
mod jxl_support;
use std::{io::Cursor, path::PathBuf, str::FromStr};

use http::{HeaderMap, HeaderValue, StatusCode, header};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use recorder::{
  media::{
    AutoOptimizeImageFormat as F, EncodeJxlOptions, MediaConfig, MediaService,
    derivative::{self, DerivativeEntry, DerivativeManifest},
  },
  storage::StorageService,
};
struct Fixture {
  root: PathBuf,
  storage: StorageService,
}
impl Fixture {
  fn new() -> Self {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("../../temp/verification/fixtures/media")
      .join(uuid::Uuid::now_v7().to_string());
    std::fs::create_dir_all(&root).unwrap();
    let storage = StorageService {
      data_dir: root.to_str().unwrap().into(),
      operator: StorageService::get_operator(root.to_str().unwrap()).unwrap(),
    };
    Self { root, storage }
  }
  async fn publish(&self, source: &str, media: &MediaService, format: F, data: bytes::Bytes) {
    let profile = media
      .derivative_plans()
      .into_iter()
      .find(|p| p.format == format)
      .map(|p| p.profile)
      .unwrap_or_else(|| derivative::profile(&recorder::media::EncodeImageOptions::Avif(recorder::media::EncodeAvifOptions::default())));
    let hash = derivative::hash(&data);
    let source_hash = derivative::hash(&self.storage.read(source).await.unwrap().to_bytes());
    let path = derivative::derivative_path(source, &profile, &source_hash, &hash, format);
    self.storage.write(path.clone(), data.clone()).await.unwrap();
    let fingerprint = derivative::fingerprint(&self.storage.stat(source).await.unwrap()).unwrap();
    let mut m = self.storage.load_derivatives(source).await.unwrap_or(DerivativeManifest {
      version: derivative::VERSION,
      source_fingerprint: fingerprint,
      source_sha256: source_hash,
      entries: Vec::new(),
    });
    m.entries.push(DerivativeEntry {
      format,
      profile,
      path,
      length: data.len() as u64,
      sha256: hash,
    });
    self
      .storage
      .write(derivative::manifest_path(source), bytes::Bytes::from(serde_json::to_vec(&m).unwrap()))
      .await
      .unwrap();
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    std::fs::remove_dir_all(&self.root).unwrap();
  }
}
fn png(w: u32, h: u32) -> bytes::Bytes {
  let image = DynamicImage::ImageRgba8(RgbaImage::from_fn(w, h, |x, y| Rgba([(x % 255) as u8, (y % 255) as u8, 128, 64])));
  let mut bytes = Cursor::new(Vec::new());
  image.write_to(&mut bytes, ImageFormat::Png).unwrap();
  bytes.into_inner().into()
}
#[tokio::test]
async fn media_delivery_negotiates_specificity_ties_availability_and_legacy() {
  let f = Fixture::new();
  let mut config = jxl_support::config();
  config.auto_optimize_formats = vec![F::Webp];
  let media = MediaService::from_config(config).await.unwrap();
  let source = "public/a.jpg";
  f.storage.write(source, b"original".as_slice().into()).await.unwrap();
  f.publish(source, &media, F::Webp, b"webp".as_slice().into()).await;
  f.publish(source, &media, F::Avif, b"avif".as_slice().into()).await;
  f.storage.write("public/a.jxl", b"unproven-legacy".as_slice().into()).await.unwrap();
  for (accept, status, mime) in [
    ("image/avif,image/webp", 200, "image/webp"),
    ("image/webp;q=0.5,image/avif", 200, "image/webp"),
    ("image/webp;q=0,image/*", 200, "image/jpeg"),
    ("application/jxl,image/jpeg", 200, "image/jpeg"),
    ("image/jxl", 406, ""),
    ("image/*;q=0,*/*;q=0", 406, ""),
    ("image/jpeg;q=1,image/webp;q=0.1", 200, "image/jpeg"),
  ] {
    let accept = headers_accept::Accept::from_str(accept).unwrap();
    let response = f
      .storage
      .serve_image(&http::Method::GET, source, &HeaderMap::new(), Some(&accept), &media, false)
      .await
      .unwrap();
    assert_eq!(response.status().as_u16(), status);
    if !mime.is_empty() {
      assert_eq!(response.headers()[header::CONTENT_TYPE], mime);
    }
    assert!(response.headers().get_all(header::VARY).iter().any(|h| h == "Accept"));
    assert_eq!(response.headers()[header::CACHE_CONTROL], "public, no-cache");
  }
  let response = f
    .storage
    .serve_image(&http::Method::GET, source, &HeaderMap::new(), None, &media, true)
    .await
    .unwrap();
  assert_eq!(response.headers()[header::CONTENT_TYPE], "image/jpeg");
  assert_eq!(response.headers()[header::CACHE_CONTROL], "private, no-cache");
  let mut malicious = f.storage.load_derivatives(source).await.unwrap();
  malicious.entries[0].path = "subscribers/1002/stolen.webp".into();
  f.storage
    .operator
    .write(&derivative::manifest_path(source), serde_json::to_vec(&malicious).unwrap())
    .await
    .unwrap();
  assert!(f.storage.load_derivatives(source).await.is_none());
  f.storage.operator.write(&derivative::manifest_path(source), "broken").await.unwrap();
  let accept = headers_accept::Accept::from_str("image/webp,image/jpeg").unwrap();
  let response = f
    .storage
    .serve_image(&http::Method::GET, source, &HeaderMap::new(), Some(&accept), &media, false)
    .await
    .unwrap();
  assert_eq!(response.headers()[header::CONTENT_TYPE], "image/jpeg");
}
#[tokio::test]
async fn media_delivery_validators_ranges_streaming_and_source_invalidation() {
  let f = Fixture::new();
  let media = MediaService::from_config(jxl_support::config()).await.unwrap();
  let source = "public/cover.jpg";
  f.storage.write(source, b"0123456789".as_slice().into()).await.unwrap();
  f.publish(source, &media, F::Webp, b"abcdefghij".as_slice().into()).await;
  let accept = headers_accept::Accept::from_str("image/webp,image/jpeg").unwrap();
  let response = f
    .storage
    .serve_image(&http::Method::GET, source, &HeaderMap::new(), Some(&accept), &media, false)
    .await
    .unwrap();
  let etag = response.headers()[header::ETAG].clone();
  assert!(!etag.to_str().unwrap().starts_with("W/"));
  assert_eq!(response.headers()[header::CONTENT_LENGTH], "10");
  let mut headers = HeaderMap::new();
  headers.insert(
    header::IF_NONE_MATCH,
    HeaderValue::from_str(&format!("\"other\", W/{}", etag.to_str().unwrap())).unwrap(),
  );
  headers.insert(header::RANGE, HeaderValue::from_static("bytes=99-100"));
  let response = f
    .storage
    .serve_image(&http::Method::GET, source, &headers, Some(&accept), &media, false)
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
  assert!(!response.headers().contains_key(header::CONTENT_RANGE));
  assert!(!response.headers().contains_key(header::CONTENT_LENGTH));
  assert_eq!(response.headers()[header::ETAG], etag);
  headers.remove(header::IF_NONE_MATCH);
  headers.insert(header::RANGE, HeaderValue::from_static("bytes=1-3"));
  headers.insert(header::IF_RANGE, etag.clone());
  let response = f
    .storage
    .serve_image(&http::Method::GET, source, &headers, Some(&accept), &media, false)
    .await
    .unwrap();
  assert_eq!(response.status(), 206);
  assert_eq!(response.headers()[header::CONTENT_LENGTH], "3");
  assert_eq!(axum::body::to_bytes(response.into_body(), 1000).await.unwrap(), "bcd");
  headers.insert(header::IF_RANGE, HeaderValue::from_static("\"other-variant\""));
  let response = f
    .storage
    .serve_image(&http::Method::GET, source, &headers, Some(&accept), &media, false)
    .await
    .unwrap();
  assert_eq!(response.status(), 200);
  assert_eq!(axum::body::to_bytes(response.into_body(), 1000).await.unwrap(), "abcdefghij");
  headers.remove(header::IF_RANGE);
  headers.insert(header::RANGE, HeaderValue::from_static("bytes=0-1,8-9"));
  let response = f
    .storage
    .serve_image(&http::Method::GET, source, &headers, Some(&accept), &media, false)
    .await
    .unwrap();
  assert_eq!(response.status(), 206);
  let length = response.headers()[header::CONTENT_LENGTH].to_str().unwrap().parse::<usize>().unwrap();
  assert_eq!(axum::body::to_bytes(response.into_body(), 4096).await.unwrap().len(), length);
  headers.insert(header::IF_RANGE, HeaderValue::from_static("invalid-if-range"));
  let response = f
    .storage
    .serve_image(&http::Method::GET, source, &headers, Some(&accept), &media, false)
    .await
    .unwrap();
  assert_eq!(response.status(), 200);
  headers.remove(header::IF_RANGE);
  headers.insert(header::IF_NONE_MATCH, HeaderValue::from_static("*"));
  let response = f
    .storage
    .serve_image(&http::Method::GET, source, &headers, Some(&accept), &media, false)
    .await
    .unwrap();
  assert_eq!(response.status(), 304);
  assert_eq!(response.headers()[header::CACHE_CONTROL], "public, no-cache");
  let mut vary = HeaderMap::new();
  vary.append(header::VARY, HeaderValue::from_static("Origin, Accept-Encoding"));
  recorder::storage::merge_vary(&mut vary, "Accept");
  recorder::storage::merge_vary(&mut vary, "accept");
  assert_eq!(
    vary.get_all(header::VARY).iter().map(|v| v.to_str().unwrap()).collect::<Vec<_>>(),
    ["Origin, Accept-Encoding", "Accept"]
  );
  let old = f.storage.load_derivatives(source).await.unwrap();
  assert!(f.storage.load_derivatives("public/cover.png").await.is_none());
  f.storage.write(source, b"new-original".as_slice().into()).await.unwrap();
  assert!(f.storage.load_derivatives(source).await.is_none());
  f.storage
    .write(derivative::manifest_path(source), bytes::Bytes::from(serde_json::to_vec(&old).unwrap()))
    .await
    .unwrap();
  assert!(f.storage.load_derivatives(source).await.is_none());
  let response = f
    .storage
    .serve_image(&http::Method::GET, source, &HeaderMap::new(), Some(&accept), &media, false)
    .await
    .unwrap();
  assert_eq!(response.headers()[header::CONTENT_TYPE], "image/jpeg");
  assert_ne!(response.headers()[header::ETAG], etag);
  assert!(response.headers()[header::ETAG].to_str().unwrap().starts_with("W/"));
}
#[tokio::test]
async fn media_delivery_budgets_old_options_and_no_feature() {
  use recorder::task::SystemTaskTrait;
  for mime in ["image/webp", "image/avif", "image/jxl"] {
    let input: recorder::task::SystemTaskInput = serde_json::from_value(serde_json::json!({
      "taskType":"optimize_image", "sourcePath":"public/input.png", "targetPath":"public/input.webp",
      "formatOptions": if mime == "image/jxl" {serde_json::json!({"mimeType":mime,"presetVersion":1})} else {serde_json::json!({"mimeType":mime,"quality":80})}
    }))
    .unwrap();
    let payload = serde_json::to_value(recorder::task::SystemTask::from_input(input, None)).unwrap();
    assert_eq!(payload["format_options"]["mime_type"], mime);
    assert!(payload["format_options"].get("mimeType").is_none());
  }
  for field in ["distance", "effort", "quality", "speed"] {
    assert!(serde_json::from_value::<EncodeJxlOptions>(serde_json::json!({field:4})).is_err());
  }
  for field in [
    "max_encoder_bytes",
    "jxl_speed",
    "jxl_effort",
    "jxl_distance",
    "jxl_quality",
    "jxl_encoder_path",
    "jxl_max_address_space_bytes",
  ] {
    assert!(serde_json::from_value::<MediaConfig>(serde_json::json!({field:4})).is_err());
  }
  let config: MediaConfig = serde_json::from_str(r#"{"execution":{"deadline_seconds":10}}"#).unwrap();
  assert_eq!(config.encode_deadline_seconds, 10);
  assert!(serde_json::from_str::<MediaConfig>(r#"{"jxl_timeout_seconds":10}"#).is_err());
  assert!(serde_json::from_str::<MediaConfig>(r#"{"jxl_timeout_seconds":10,"encode_deadline_seconds":10}"#).is_err());
  for (group, field) in [
    ("limits", "input_bytes"),
    ("limits", "pixels"),
    ("limits", "decode_bytes"),
    ("execution", "working_set_bytes"),
    ("execution", "concurrency"),
  ] {
    let config: MediaConfig = serde_json::from_value(serde_json::json!({group:{field:0}})).unwrap();
    assert!(MediaService::from_config(config).await.is_err());
  }
  let mut config = jxl_support::config();
  config.max_pixels = 10;
  let media = MediaService::from_config(config).await.unwrap();
  assert!(media.optimize_image_to_webp("a.png", png(7, 5), None).await.is_err());
  assert_eq!(media.available_permits(), 1);
  let mut config = jxl_support::config();
  config.max_input_bytes = 10;
  let media = MediaService::from_config(config).await.unwrap();
  assert!(media.optimize_image_to_webp("a.png", png(7, 5), None).await.is_err());
  #[cfg(not(feature = "jxl"))]
  {
    let mut config = jxl_support::config();
    config.auto_optimize_formats = vec![F::Jxl];
    assert!(MediaService::from_config(config).await.is_err());
  }
}
#[cfg(feature = "jxl")]
#[tokio::test]
async fn media_delivery_jxl_profile_and_collaborative_stop_hold_permit() {
  let mut config = jxl_support::config();
  config.auto_optimize_formats = vec![F::Jxl, F::Webp];
  let media = std::sync::Arc::new(MediaService::from_config(config).await.unwrap());
  let f = Fixture::new();
  f.storage.write("public/a.jpg", png(32, 32)).await.unwrap();
  f.publish("public/a.jpg", &media, F::Webp, b"webp".as_slice().into()).await;
  f.publish("public/a.jpg", &media, F::Jxl, b"jxl".as_slice().into()).await;
  for (header, expected) in [
    ("image/webp,image/jxl", "image/jxl"),
    ("image/jxl,image/webp", "image/jxl"),
    ("image/jxl;q=0.4,image/webp;q=0.8", "image/webp"),
    ("image/jxl;q=0,image/*", "image/webp"),
    ("image/*", "image/webp"),
    ("application/jxl,image/webp", "image/webp"),
  ] {
    let accept = headers_accept::Accept::from_str(header).unwrap();
    let response = f
      .storage
      .serve_image(&http::Method::GET, "public/a.jpg", &HeaderMap::new(), Some(&accept), &media, false)
      .await
      .unwrap();
    assert_eq!(response.headers()[header::CONTENT_TYPE], expected);
  }
  let data = png(1024, 1024);
  let service = media.clone();
  let handle = tokio::spawn(async move { service.optimize_image_to_jxl("big.png", data, None).await });
  tokio::time::timeout(std::time::Duration::from_secs(5), async {
    while media.available_permits() != 0 {
      tokio::task::yield_now().await;
    }
  })
  .await
  .unwrap();
  handle.abort();
  let _ = handle.await;
  tokio::time::timeout(std::time::Duration::from_secs(10), async {
    while media.available_permits() != 1 {
      tokio::task::yield_now().await;
    }
  })
  .await
  .unwrap();
}

#[tokio::test]
async fn media_delivery_orientation_and_unsupported_profile_preserve_input() {
  use image::{GenericImageView, ImageEncoder};
  let config = jxl_support::config();
  let image = image::RgbImage::from_fn(3, 2, |x, y| image::Rgb([(x * 40) as u8, (y * 90) as u8, 100]));
  let mut encoded = Vec::new();
  // TIFF IFD0 orientation 6 (rotate 90 degrees clockwise).
  let exif = vec![b'I', b'I', 42, 0, 8, 0, 0, 0, 1, 0, 0x12, 1, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0];
  let mut encoder = image::codecs::png::PngEncoder::new(&mut encoded);
  encoder.set_exif_metadata(exif).unwrap();
  encoder.write_image(image.as_raw(), 3, 2, image::ExtendedColorType::Rgb8).unwrap();
  let normalized = recorder::media::decode(&encoded, &config).unwrap();
  assert_eq!(normalized.dimensions(), (2, 3));
  let mut profiled = Vec::new();
  let mut encoder = image::codecs::png::PngEncoder::new(&mut profiled);
  encoder.set_icc_profile(b"unclassified-profile".to_vec()).unwrap();
  encoder.write_image(image.as_raw(), 3, 2, image::ExtendedColorType::Rgb8).unwrap();
  let original = profiled.clone();
  let error = MediaService::from_config(config)
    .await
    .unwrap()
    .optimize_image_to_webp("profile.png", profiled, None)
    .await
    .unwrap_err();
  assert!(error.to_string().contains("profile"));
  assert!(!original.is_empty());
  let encode_metadata = |mut info: png::Info<'static>| {
    info.width = 3;
    info.height = 2;
    info.color_type = png::ColorType::Rgb;
    info.bit_depth = png::BitDepth::Eight;
    let cicp = info.coding_independent_code_points;
    let animated = info.animation_control.take().is_some();
    let mut encoded = Vec::new();
    let mut encoder = png::Encoder::with_info(&mut encoded, info).unwrap();
    if animated {
      encoder.set_animated(1, 0).unwrap();
    }
    let mut writer = encoder.write_header().unwrap();
    // png 0.18 parses cICP but does not serialize
    // Info.coding_independent_code_points.
    if let Some(cicp) = cicp {
      writer
        .write_chunk(
          png::chunk::cICP,
          &[
            cicp.color_primaries,
            cicp.transfer_function,
            cicp.matrix_coefficients,
            u8::from(cicp.is_video_full_range_image),
          ],
        )
        .unwrap();
    }
    writer.write_image_data(image.as_raw()).unwrap();
    drop(writer);
    encoded
  };
  let config = jxl_support::config();
  let mut hdr = png::Info::default();
  hdr.coding_independent_code_points = Some(png::CodingIndependentCodePoints {
    color_primaries: 9,
    transfer_function: 16,
    matrix_coefficients: 0,
    is_video_full_range_image: true,
  });
  let mut linear = png::Info::default();
  linear.source_gamma = Some(png::ScaledFloat::new(1.0));
  let mut animated = png::Info::default();
  animated.animation_control = Some(png::AnimationControl { num_frames: 1, num_plays: 0 });
  for (category, info) in [("hdr", hdr), ("linear", linear), ("animation", animated)] {
    let encoded = encode_metadata(info);
    let before = encoded.clone();
    assert!(recorder::media::decode(&encoded, &config).is_err(), "{category} must be rejected");
    assert_eq!(encoded, before);
  }
  let mut srgb = png::Info::default();
  srgb.srgb = Some(png::SrgbRenderingIntent::Perceptual);
  assert_eq!(recorder::media::decode(&encode_metadata(srgb), &config).unwrap().dimensions(), (3, 2));
}

#[tokio::test]
async fn media_delivery_previous_qualified_profile_remains_negotiable_during_backfill() {
  let formats = if cfg!(feature = "jxl") { vec![F::Webp, F::Jxl] } else { vec![F::Webp] };
  for format in formats {
    let f = Fixture::new();
    let media = MediaService::from_config(MediaConfig {
      auto_optimize_formats: if format == F::Jxl { vec![F::Jxl, F::Webp] } else { vec![F::Webp] },
      ..Default::default()
    })
    .await
    .unwrap();
    let source = "public/migration.png";
    f.storage.write(source, png(16, 16)).await.unwrap();
    let options = media.derivative_plans().into_iter().find(|plan| plan.format == format).unwrap().options;
    let (policy, bytes) = match format {
      F::Webp => (
        "webp-0.3.1-cover1600-premultiplied-lanczos3-v2",
        media.optimize_cover_to_webp(png(16, 16), None).await.unwrap(),
      ),
      F::Jxl => (
        "cover-jxl-v1-libjxl-0.12.0-1600-premultiplied-lanczos3-alpha0-ac-dc0",
        media.optimize_cover_to_jxl(png(16, 16), None).await.unwrap(),
      ),
      F::Avif => unreachable!(),
    };
    #[cfg(feature = "jxl")]
    if format == F::Jxl {
      assert_eq!(jxl_support::decode(&bytes).dimensions(), (16, 16));
    }
    let profile = derivative::hash(
      format!(
        "{policy}:{}",
        if format == F::Jxl {
          r#"{"mime_type":"image/jxl","distance":2.8,"effort":7}"#.into()
        } else {
          serde_json::to_string(&options).unwrap()
        }
      )
      .as_bytes(),
    );
    let content_hash = derivative::hash(&bytes);
    let source_hash = derivative::hash(&f.storage.read(source).await.unwrap().to_bytes());
    let path = derivative::derivative_path(source, &profile, &source_hash, &content_hash, format);
    f.storage.write(&path, bytes.clone()).await.unwrap();
    let manifest = DerivativeManifest {
      version: 2,
      source_fingerprint: derivative::fingerprint(&f.storage.stat(source).await.unwrap()).unwrap(),
      source_sha256: source_hash,
      entries: vec![DerivativeEntry {
        profile,
        format,
        path: path.clone(),
        length: bytes.len() as u64,
        sha256: content_hash,
      }],
    };
    f.storage
      .write(derivative::manifest_path(source), serde_json::to_vec(&manifest).unwrap().into())
      .await
      .unwrap();
    assert!(f.storage.load_derivatives(source).await.is_some());
    assert_eq!(f.storage.missing_derivatives(source, &media).await.len(), media.derivative_plans().len());
    let response = f
      .storage
      .serve_image(
        &http::Method::GET,
        source,
        &HeaderMap::new(),
        Some(&headers_accept::Accept::from_str(format.mime()).unwrap()),
        &media,
        false,
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], format.mime());
    assert_eq!(f.storage.read(path).await.unwrap().to_bytes(), bytes);
  }
}
