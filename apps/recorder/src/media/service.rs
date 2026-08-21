use std::{
  io::Cursor,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
};

use bytes::Bytes;
use image::{DynamicImage, ImageDecoder, ImageEncoder, ImageFormat, ImageReader, codecs::avif::AvifEncoder};
use quirks_path::Path;

use crate::{
  errors::{RecorderError, RecorderResult},
  media::{EncodeAvifOptions, EncodeImageOptions, EncodeJxlOptions, EncodeWebpOptions, MediaConfig},
};

#[derive(Debug)]
pub struct MediaService {
  pub config: MediaConfig,
  executor: super::executor::MediaExecutor,
}
struct CancelGuard(Arc<AtomicBool>);
impl Drop for CancelGuard {
  fn drop(&mut self) {
    self.0.store(true, Ordering::Release);
  }
}

impl MediaService {
  pub async fn from_config(config: MediaConfig) -> RecorderResult<Self> {
    config.validate()?;
    let executor = super::executor::MediaExecutor::new(&config)?;
    Ok(Self { config, executor })
  }
  pub async fn shutdown(&self) -> RecorderResult<()> {
    self.executor.shutdown().await
  }
  pub fn is_legacy_image_format(&self, ext: &str) -> bool {
    matches!(ext, "jpeg" | "jpg" | "png")
  }
  #[cfg(any(test, feature = "test-utils"))]
  pub fn available_permits(&self) -> usize {
    self.executor.available_permits()
  }
  pub async fn optimize_image_to_webp(&self, _path: impl AsRef<Path>, data: impl Into<Bytes>, options: Option<EncodeWebpOptions>) -> RecorderResult<Bytes> {
    self.encode(data.into(), EncodeImageOptions::Webp(options.unwrap_or_default()), false).await
  }
  pub async fn optimize_cover_to_webp(&self, data: Bytes, options: Option<EncodeWebpOptions>) -> RecorderResult<Bytes> {
    self.encode(data, EncodeImageOptions::Webp(options.unwrap_or_default()), true).await
  }
  pub async fn optimize_image_to_avif(&self, _path: impl AsRef<Path>, data: Bytes, options: Option<EncodeAvifOptions>) -> RecorderResult<Bytes> {
    self.encode(data, EncodeImageOptions::Avif(options.unwrap_or_default()), false).await
  }
  pub async fn optimize_image_to_jxl(&self, _path: impl AsRef<Path>, data: Bytes, options: Option<EncodeJxlOptions>) -> RecorderResult<Bytes> {
    self.encode(data, EncodeImageOptions::Jxl(options.unwrap_or_default()), false).await
  }
  pub async fn optimize_cover_to_jxl(&self, data: Bytes, options: Option<EncodeJxlOptions>) -> RecorderResult<Bytes> {
    self.encode(data, EncodeImageOptions::Jxl(options.unwrap_or_default()), true).await
  }
  async fn encode(&self, data: Bytes, options: EncodeImageOptions, cover: bool) -> RecorderResult<Bytes> {
    match &options {
      EncodeImageOptions::Webp(o) => validate_quality(o.quality.unwrap_or(self.config.webp_quality))?,
      EncodeImageOptions::Jxl(o) => {
        if !cfg!(feature = "jxl") {
          return Err(invalid_options("jxl feature is not enabled"));
        }
        if o.preset_version != 1 {
          return Err(invalid_options("JXL preset_version is unsupported; use preset version 1 (Balanced 77)"));
        }
      }
      EncodeImageOptions::Avif(o) => {
        if o.quality.unwrap_or(self.config.avif_quality) > 100
          || !(1..=10).contains(&o.speed.unwrap_or(self.config.avif_speed))
          || o.threads.unwrap_or(self.config.avif_threads) == 0
        {
          return Err(invalid_options("Invalid AVIF options"));
        }
      }
    }
    let config = self.config.clone();
    let cancelled = Arc::new(AtomicBool::new(false));
    let _guard = CancelGuard(cancelled.clone());
    // Metadata is inspected without allocating a raster. The decoded pixels and
    // codec workspace are created only after the shared admission is acquired.
    let estimated = estimate(&data, &config, cover)?;
    let result = self.executor.execute(
      estimated,
      cancelled,
      Box::new(move |cancelled| {
        checkpoint(cancelled)?;
        let image = decode(&data, &config)?;
        drop(data);
        checkpoint(cancelled)?;
        let image = if cover { normalize_cover(image, &config)? } else { image };
        checkpoint(cancelled)?;
        let output = match options {
          EncodeImageOptions::Webp(o) => encode_webp(image, o.quality.unwrap_or(config.webp_quality))?,
          #[cfg(feature = "jxl")]
          EncodeImageOptions::Jxl(_) => super::jxl::encode(image)?,
          EncodeImageOptions::Avif(o) => {
            let mut output = Vec::new();
            AvifEncoder::new_with_speed_quality(&mut output, o.speed.unwrap_or(config.avif_speed), o.quality.unwrap_or(config.avif_quality))
              .with_num_threads(Some(o.threads.unwrap_or(config.avif_threads) as usize))
              .write_image(image.as_bytes(), image.width(), image.height(), image.color().into())?;
            output
          }
          #[cfg(not(feature = "jxl"))]
          EncodeImageOptions::Jxl(_) => return Err(invalid_options("jxl feature is not enabled")),
        };
        checkpoint(cancelled)?;
        if output.is_empty() || output.len() as u64 > config.max_output_bytes {
          return Err(invalid_options("Image output byte budget exceeded"));
        }
        Ok(output.into())
      }),
    );
    tokio::time::timeout(std::time::Duration::from_secs(self.config.encode_deadline_seconds), result)
      .await
      .map_err(|_| invalid_options("Media encoding deadline expired; synchronous codec may still be running"))?
  }
}
fn checkpoint(cancelled: &AtomicBool) -> RecorderResult<()> {
  if cancelled.load(Ordering::Acquire) {
    Err(invalid_options("Media encoding cancelled"))
  } else {
    Ok(())
  }
}
fn estimate(data: &[u8], config: &MediaConfig, cover: bool) -> RecorderResult<u64> {
  if data.len() as u64 > config.max_input_bytes {
    return Err(invalid_options("Image input byte budget exceeded"));
  }
  let format = image::guess_format(data)?;
  if !matches!(format, ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP) {
    return Err(invalid_options("Image skipped: unsupported static SDR format"));
  }
  let mut reader = ImageReader::with_format(Cursor::new(data), format);
  let mut limits = image::Limits::default();
  limits.max_alloc = Some(config.max_decode_bytes);
  limits.max_image_width = Some(config.max_dimension);
  limits.max_image_height = Some(config.max_dimension);
  reader.limits(limits);
  let (w, h) = reader.into_dimensions()?;
  let pixels = u64::from(w) * u64::from(h);
  if pixels > config.max_pixels || pixels.saturating_mul(4) > config.max_decode_bytes {
    return Err(invalid_options("Image pixel/decode budget exceeded"));
  }
  let normalized = if cover && w.max(h) > 1600 {
    pixels * 1600 * 1600 / u64::from(w.max(h)).pow(2)
  } else {
    pixels
  };
  // An admission estimate, not an OS RSS ceiling. One budget covers queued
  // source bytes, decode/resize rasters, both codecs and bounded output.
  Ok(
    (data.len() as u64)
      .saturating_add(pixels.saturating_mul(8))
      .saturating_add(normalized.saturating_mul(128))
      .saturating_add(config.max_output_bytes),
  )
}
pub fn normalize_cover(image: DynamicImage, _config: &MediaConfig) -> RecorderResult<DynamicImage> {
  // Accepted samples are sRGB. Composite encoded sRGB samples on white before
  // Lanczos resize, so invisible RGB never contaminates filtered cover edges.
  let rgba = image.into_rgba8();
  let rgb = image::RgbImage::from_fn(rgba.width(), rgba.height(), |x, y| {
    let p = rgba.get_pixel(x, y);
    let a = u32::from(p[3]);
    image::Rgb(std::array::from_fn(|c| ((u32::from(p[c]) * a + 255 * (255 - a) + 127) / 255) as u8))
  });
  let image = DynamicImage::ImageRgb8(rgb);
  let (w, h) = (image.width(), image.height());
  let long = w.max(h);
  if long <= 1600 {
    return Ok(image);
  }
  let scaled = |v: u32| ((u64::from(v) * 1600 + u64::from(long) / 2) / u64::from(long)).max(1) as u32;
  Ok(image.resize_exact(scaled(w), scaled(h), image::imageops::FilterType::Lanczos3))
}
fn encode_webp(image: DynamicImage, quality: f32) -> RecorderResult<Vec<u8>> {
  validate_quality(quality)?;
  let (w, h) = (image.width(), image.height());
  // The simple libwebp API uses thread_level=0. No task creates a codec pool.
  Ok(if image.color().has_alpha() {
    webp::Encoder::from_rgba(image.into_rgba8().as_raw(), w, h).encode(quality).to_vec()
  } else {
    webp::Encoder::from_rgb(image.into_rgb8().as_raw(), w, h).encode(quality).to_vec()
  })
}

pub fn decode(data: &[u8], config: &MediaConfig) -> RecorderResult<DynamicImage> {
  if data.len() as u64 > config.max_input_bytes {
    return Err(invalid_options("Image input byte budget exceeded"));
  }
  let format = image::guess_format(data)?;
  if !matches!(format, ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP) {
    return Err(invalid_options("Image skipped: unsupported static SDR format"));
  }
  if format == ImageFormat::Png {
    let metadata = png::Decoder::new_with_limits(
      Cursor::new(data),
      png::Limits {
        bytes: usize::try_from(config.max_decode_bytes).unwrap_or(usize::MAX),
      },
    )
    .read_info()
    .map_err(|_| invalid_options("Image skipped: invalid or oversized PNG metadata"))?;
    let info = metadata.info();
    if info.animation_control.is_some() {
      return Err(invalid_options("Image skipped: animation"));
    }
    // Eight-bit samples can still declare HDR or a transfer function that we
    // cannot preserve. Until color management exists, accept only implicit or
    // explicitly declared sRGB PNG color interpretation.
    if info.coding_independent_code_points.is_some()
      || info.mastering_display_color_volume.is_some()
      || info.content_light_level.is_some()
      || (info.srgb.is_none() && (info.gamma().is_some() || info.chromaticities().is_some()))
    {
      return Err(invalid_options("Image skipped: unsupported color profile or HDR metadata"));
    }
  }
  let mut limits = image::Limits::default();
  limits.max_image_width = Some(config.max_dimension);
  limits.max_image_height = Some(config.max_dimension);
  limits.max_alloc = Some(config.max_decode_bytes);
  let mut reader = ImageReader::with_format(Cursor::new(data), format);
  reader.limits(limits.clone());
  let mut decoder = reader.into_decoder()?;
  let (w, h) = decoder.dimensions();
  if u64::from(w).checked_mul(u64::from(h)).is_none_or(|pixels| pixels > config.max_pixels) || decoder.total_bytes() > config.max_decode_bytes {
    return Err(invalid_options("Image pixel/decode budget exceeded"));
  }
  // Until a color-management contract exists, reject all embedded profiles
  // conservatively.
  if decoder.icc_profile()?.is_some() || !matches!(decoder.color_type(), image::ColorType::Rgb8 | image::ColorType::Rgba8) {
    return Err(invalid_options("Image skipped: unsupported color profile or bit depth"));
  }
  if format == ImageFormat::WebP && image::codecs::webp::WebPDecoder::new(Cursor::new(data))?.has_animation() {
    return Err(invalid_options("Image skipped: animation"));
  }
  let orientation = decoder.orientation()?;
  let mut image = DynamicImage::from_decoder(decoder)?;
  image.apply_orientation(orientation);
  Ok(image)
}
pub fn invalid_options(message: &str) -> RecorderError {
  RecorderError::Whatever {
    message: message.into(),
    source: None.into(),
  }
}
fn validate_quality(quality: f32) -> RecorderResult<()> {
  if !quality.is_finite() || !(0.0..=100.0).contains(&quality) {
    return Err(invalid_options("Image quality must be finite and between 0 and 100"));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn formats_contend_for_the_same_owned_admission_budget() {
    // Match the executor lifecycle test's Chili global pool configuration.
    let config = MediaConfig {
      encode_concurrency: 2,
      encode_working_set_bytes: 2 * 1024 * 1024,
      max_output_bytes: 16 * 1024,
      ..Default::default()
    };
    let service = Arc::new(MediaService::from_config(config.clone()).await.unwrap());
    let (started, ready) = tokio::sync::oneshot::channel();
    let (release, blocked) = std::sync::mpsc::channel();
    let owner = service.clone();
    let held = tokio::spawn(async move {
      owner
        .executor
        .execute(
          owner.config.encode_working_set_bytes,
          Arc::new(AtomicBool::new(false)),
          Box::new(move |_| {
            let _ = started.send(());
            blocked.recv().unwrap();
            Ok(Bytes::new())
          }),
        )
        .await
    });
    ready.await.unwrap();
    let mut data = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(image::RgbImage::from_pixel(16, 12, image::Rgb([80, 40, 120])))
      .write_to(&mut data, ImageFormat::Png)
      .unwrap();
    let data = Bytes::from(data.into_inner());
    let webp = service.optimize_cover_to_webp(data.clone(), None).await.unwrap_err();
    assert!(webp.to_string().contains("admission budget unavailable"));
    #[cfg(feature = "jxl")]
    {
      let jxl = service.optimize_cover_to_jxl(data.clone(), None).await.unwrap_err();
      assert!(jxl.to_string().contains("admission budget unavailable"));
    }
    release.send(()).unwrap();
    held.await.unwrap().unwrap();
    service.shutdown().await.unwrap();
    // Both recipes fit individually; their rejection above was contention,
    // rather than an oversized-input or unsupported-format error.
    let service = MediaService::from_config(config).await.unwrap();
    service.optimize_cover_to_webp(data.clone(), None).await.unwrap();
    #[cfg(feature = "jxl")]
    service.optimize_cover_to_jxl(data, None).await.unwrap();
    service.shutdown().await.unwrap();
  }
}
