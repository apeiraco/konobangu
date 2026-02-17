use std::io::Cursor;

use bytes::Bytes;
use image::{GenericImageView, ImageEncoder, ImageReader, codecs::avif::AvifEncoder};
use quirks_path::Path;
use snafu::ResultExt;

use crate::{
    errors::{RecorderError, RecorderResult},
    media::{EncodeAvifOptions, EncodeJxlOptions, EncodeWebpOptions, MediaConfig},
};

#[derive(Debug)]
pub struct MediaService {
    pub config: MediaConfig,
}

impl MediaService {
    pub async fn from_config(config: MediaConfig) -> RecorderResult<Self> {
        Ok(Self { config })
    }

    pub fn is_legacy_image_format(&self, ext: &str) -> bool {
        matches!(ext, "jpeg" | "jpg" | "png")
    }

    pub async fn optimize_image_to_webp(
        &self,
        path: impl AsRef<Path>,
        data: impl Into<Bytes>,
        options: Option<EncodeWebpOptions>,
    ) -> RecorderResult<Bytes> {
        let quality = options
            .and_then(|o| o.quality)
            .unwrap_or(self.config.webp_quality);

        let data = data.into();

        tokio::task::spawn_blocking(move || -> RecorderResult<Bytes> {
            let cursor = Cursor::new(data);
            let image_reader = ImageReader::new(cursor).with_guessed_format()?;

            let img = image_reader.decode()?;

            let (width, height) = (img.width(), img.height());

            let color = img.color();

            let webp_data = if color.has_alpha() {
                let rgba_image = img.into_rgba8();

                let encoder = webp::Encoder::from_rgba(&rgba_image, width, height);

                encoder.encode(quality)
            } else {
                let rgba_image = img.into_rgb8();

                let encoder = webp::Encoder::from_rgb(&rgba_image, width, height);

                encoder.encode(quality)
            };

            Ok(Bytes::from(webp_data.to_vec()))
        })
        .await
        .with_whatever_context::<_, String, RecorderError>(|_| {
            format!(
                "failed to spawn blocking task to optimize legacy image to webp: {}",
                path.as_ref().display()
            )
        })?
    }

    pub async fn optimize_image_to_avif(
        &self,
        path: impl AsRef<Path>,
        data: Bytes,
        options: Option<EncodeAvifOptions>,
    ) -> RecorderResult<Bytes> {
        let quality = options
            .as_ref()
            .and_then(|o| o.quality)
            .unwrap_or(self.config.avif_quality);
        let speed = options
            .as_ref()
            .and_then(|o| o.speed)
            .unwrap_or(self.config.avif_speed);
        let threads = options
            .as_ref()
            .and_then(|o| o.threads)
            .unwrap_or(self.config.avif_threads);

        tokio::task::spawn_blocking(move || -> RecorderResult<Bytes> {
            let mut buf = vec![];

            {
                let cursor = Cursor::new(data);
                let image_reader = ImageReader::new(cursor).with_guessed_format()?;

                let img = image_reader.decode()?;

                let (width, height) = img.dimensions();
                let color_type = img.color();
                let encoder = AvifEncoder::new_with_speed_quality(&mut buf, speed, quality)
                    .with_num_threads(Some(threads as usize));

                encoder.write_image(img.as_bytes(), width, height, color_type.into())?;
            }

            Ok(Bytes::from(buf))
        })
        .await
        .with_whatever_context::<_, String, RecorderError>(|_| {
            format!(
                "failed to spawn blocking task to optimize legacy image to avif: {}",
                path.as_ref().display()
            )
        })?
    }

    #[cfg(feature = "jxl")]
    pub async fn optimize_image_to_jxl(
        &self,
        path: impl AsRef<Path>,
        data: Bytes,
        options: Option<EncodeJxlOptions>,
    ) -> RecorderResult<Bytes> {
        let quality = options
            .as_ref()
            .and_then(|o| o.quality)
            .unwrap_or(self.config.jxl_quality);
        let speed = options
            .as_ref()
            .and_then(|o| o.speed)
            .unwrap_or(self.config.jxl_speed);
        tokio::task::spawn_blocking(move || -> RecorderResult<Bytes> {
            use jxl_encoder::api::{LossyConfig, PixelLayout, Quality};
            let cursor = Cursor::new(data);
            let image_reader = ImageReader::new(cursor).with_guessed_format()?;

            let image = image_reader.decode()?;
            let (width, height) = image.dimensions();

            let color = image.color();
            // jxl_encoder effort 1-10: higher = slower/better. Map speed 0-9 -> effort 1-10.
            let effort = (speed + 1).clamp(1, 10);
            let quality_spec = Quality::Percent((quality.round() as u32).min(100));
            let config = LossyConfig::from_quality(quality_spec)?
                .with_effort(effort);

            let jxl_bytes = if color.has_alpha() {
                let sample = image.into_rgba8();
                config.encode(sample.as_raw(), width, height, PixelLayout::Rgba8)?
            } else {
                let sample = image.into_rgb8();
                config.encode(sample.as_raw(), width, height, PixelLayout::Rgb8)?
            };

            Ok(Bytes::from(jxl_bytes))
        })
        .await
        .with_whatever_context::<_, String, RecorderError>(|_| {
            format!(
                "failed to spawn blocking task to optimize image to jxl: {}",
                path.as_ref().display()
            )
        })?
    }

    #[cfg(not(feature = "jxl"))]
    pub async fn optimize_image_to_jxl(
        &self,
        _path: impl AsRef<Path>,
        _data: Bytes,
        _options: Option<EncodeJxlOptions>,
    ) -> RecorderResult<Bytes> {
        Err(RecorderError::Whatever {
            message: "jxl feature is not enabled".to_string(),
            source: None.into(),
        })
    }
}
