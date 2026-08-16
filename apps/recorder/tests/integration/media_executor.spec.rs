//! Production adapter and shared lifecycle; deterministic scheduling is covered
//! inside the executor module with controlled tasks rather than native timing.
use std::io::Cursor;

use bytes::Bytes;
use image::{DynamicImage, ImageFormat};
use recorder::media::{MediaConfig, MediaService, normalize_cover};
fn input() -> Bytes {
  let image = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(16, 12, |x, y| image::Rgba([(x * 15) as u8, (y * 20) as u8, 128, 64])));
  let mut out = Cursor::new(Vec::new());
  image.write_to(&mut out, ImageFormat::Png).unwrap();
  out.into_inner().into()
}
#[tokio::test]
async fn shared_formats_white_covers_general_alpha_and_shutdown() {
  let service = MediaService::from_config(MediaConfig::default()).await.unwrap();
  let webp = service.optimize_cover_to_webp(input(), None).await.unwrap();
  assert!(image::load_from_memory(&webp).unwrap().into_rgba8().pixels().all(|p| p[3] == 255));
  let alpha = service.optimize_image_to_webp("input.png", input(), None).await.unwrap();
  assert!(image::load_from_memory(&alpha).unwrap().into_rgba8().pixels().all(|p| p[3] == 64));
  #[cfg(feature = "jxl")]
  {
    let jxl = service.optimize_cover_to_jxl(input(), None).await.unwrap();
    assert!(jxl.starts_with(&[255, 10]));
    assert!(
      service
        .optimize_image_to_jxl("alpha.png", input(), None)
        .await
        .unwrap_err()
        .to_string()
        .contains("RGBA")
    );
  }
  service.shutdown().await.unwrap();
  service.shutdown().await.unwrap();
  assert!(service.optimize_cover_to_webp(input(), None).await.is_err());
}
#[test]
fn white_composite_before_resize_ignores_hidden_color_and_never_upscales() {
  let config = MediaConfig::default();
  let a = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(1800, 2, |x, _| {
    if x < 900 { image::Rgba([255, 0, 0, 0]) } else { image::Rgba([0, 0, 0, 255]) }
  }));
  let b = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(1800, 2, |x, _| {
    if x < 900 { image::Rgba([0, 255, 0, 0]) } else { image::Rgba([0, 0, 0, 255]) }
  }));
  let normalized = normalize_cover(a, &config).unwrap();
  assert_eq!(normalized, normalize_cover(b, &config).unwrap());
  assert_eq!((normalized.width(), normalized.height()), (1600, 2));
  let small = normalize_cover(image::load_from_memory(&input()).unwrap(), &config).unwrap();
  assert_eq!((small.width(), small.height()), (16, 12));
  assert_eq!(small.into_rgb8().get_pixel(0, 0).0, [191, 191, 223]);
}
