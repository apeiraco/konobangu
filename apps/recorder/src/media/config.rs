use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::errors::{RecorderError, RecorderResult};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, TS)]
#[ts(rename_all = "camelCase")]
pub enum AutoOptimizeImageFormat {
  #[serde(rename = "image/webp")]
  Webp,
  #[serde(rename = "image/avif")]
  Avif,
  #[serde(rename = "image/jxl")]
  Jxl,
}
impl AutoOptimizeImageFormat {
  pub fn extension(self) -> &'static str {
    match self {
      Self::Webp => "webp",
      Self::Avif => "avif",
      Self::Jxl => "jxl",
    }
  }
  pub fn mime(self) -> &'static str {
    match self {
      Self::Webp => "image/webp",
      Self::Avif => "image/avif",
      Self::Jxl => "image/jxl",
    }
  }
}
#[derive(Clone, Debug, Serialize, Deserialize, Default, TS, PartialEq)]
#[ts(rename_all = "camelCase")]
pub struct EncodeWebpOptions {
  pub quality: Option<f32>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default, TS, PartialEq)]
#[ts(rename_all = "camelCase")]
pub struct EncodeAvifOptions {
  pub quality: Option<u8>,
  pub speed: Option<u8>,
  pub threads: Option<u8>,
}
/// Versioned business preset; legacy libjxl numeric knobs have no equivalent.
#[derive(Clone, Debug, Serialize, TS, PartialEq)]
#[ts(rename_all = "camelCase")]
pub struct EncodeJxlOptions {
  pub preset_version: u8,
}
impl<'de> Deserialize<'de> for EncodeJxlOptions {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Wire {
      #[serde(alias = "presetVersion")]
      preset_version: u8,
    }
    Wire::deserialize(deserializer).map(|wire| Self {
      preset_version: wire.preset_version,
    })
  }
}
impl Default for EncodeJxlOptions {
  fn default() -> Self {
    Self { preset_version: 1 }
  }
}
#[derive(Clone, Debug, Serialize, TS, PartialEq)]
#[ts(tag = "mimeType")]
#[serde(tag = "mime_type")]
pub enum EncodeImageOptions {
  #[serde(rename = "image/webp")]
  Webp(EncodeWebpOptions),
  #[serde(rename = "image/avif")]
  Avif(EncodeAvifOptions),
  #[serde(rename = "image/jxl")]
  Jxl(EncodeJxlOptions),
}

impl<'de> Deserialize<'de> for EncodeImageOptions {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    #[derive(Deserialize)]
    struct Wire {
      // Generated GraphQL input uses camel case; durable jobs retain snake case.
      #[serde(rename = "mime_type", alias = "mimeType")]
      format: AutoOptimizeImageFormat,
      #[serde(flatten)]
      options: serde_json::Map<String, serde_json::Value>,
    }
    let wire = Wire::deserialize(deserializer)?;
    let options = serde_json::Value::Object(wire.options);
    match wire.format {
      AutoOptimizeImageFormat::Webp => serde_json::from_value(options).map(Self::Webp),
      AutoOptimizeImageFormat::Avif => serde_json::from_value(options).map(Self::Avif),
      AutoOptimizeImageFormat::Jxl => serde_json::from_value(options).map(Self::Jxl),
    }
    .map_err(serde::de::Error::custom)
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(from = "MediaWire", into = "MediaWire")]
pub struct MediaConfig {
  pub webp_quality: f32,
  pub avif_quality: u8,
  pub avif_speed: u8,
  pub avif_threads: u8,
  pub encode_deadline_seconds: u64,
  pub encode_queue_capacity: usize,
  pub max_output_bytes: u64,
  pub auto_optimize_formats: Vec<AutoOptimizeImageFormat>,
  pub max_input_bytes: u64,
  pub max_dimension: u32,
  pub max_pixels: u64,
  pub max_decode_bytes: u64,
  pub encode_working_set_bytes: u64,
  pub encode_concurrency: usize,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct MediaWire {
  auto_optimize_formats: Vec<AutoOptimizeImageFormat>,
  encoding: EncodingWire,
  execution: ExecutionWire,
  limits: LimitsWire,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct EncodingWire {
  webp: WebpWire,
  avif: AvifWire,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct WebpWire {
  quality: f32,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct AvifWire {
  quality: u8,
  speed: u8,
  threads: u8,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ExecutionWire {
  deadline_seconds: u64,
  queue_capacity: usize,
  working_set_bytes: u64,
  concurrency: usize,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct LimitsWire {
  input_bytes: u64,
  output_bytes: u64,
  dimension: u32,
  pixels: u64,
  decode_bytes: u64,
}
impl Default for MediaWire {
  fn default() -> Self {
    MediaConfig::default().into()
  }
}
impl Default for EncodingWire {
  fn default() -> Self {
    MediaWire::default().encoding
  }
}
impl Default for WebpWire {
  fn default() -> Self {
    EncodingWire::default().webp
  }
}
impl Default for AvifWire {
  fn default() -> Self {
    EncodingWire::default().avif
  }
}
impl Default for ExecutionWire {
  fn default() -> Self {
    MediaWire::default().execution
  }
}
impl Default for LimitsWire {
  fn default() -> Self {
    MediaWire::default().limits
  }
}
impl From<MediaWire> for MediaConfig {
  fn from(w: MediaWire) -> Self {
    Self {
      webp_quality: w.encoding.webp.quality,
      avif_quality: w.encoding.avif.quality,
      avif_speed: w.encoding.avif.speed,
      avif_threads: w.encoding.avif.threads,
      encode_deadline_seconds: w.execution.deadline_seconds,
      encode_queue_capacity: w.execution.queue_capacity,
      encode_working_set_bytes: w.execution.working_set_bytes,
      encode_concurrency: w.execution.concurrency,
      max_input_bytes: w.limits.input_bytes,
      max_output_bytes: w.limits.output_bytes,
      max_dimension: w.limits.dimension,
      max_pixels: w.limits.pixels,
      max_decode_bytes: w.limits.decode_bytes,
      auto_optimize_formats: w.auto_optimize_formats,
    }
  }
}
impl From<MediaConfig> for MediaWire {
  fn from(c: MediaConfig) -> Self {
    Self {
      auto_optimize_formats: c.auto_optimize_formats,
      encoding: EncodingWire {
        webp: WebpWire { quality: c.webp_quality },
        avif: AvifWire {
          quality: c.avif_quality,
          speed: c.avif_speed,
          threads: c.avif_threads,
        },
      },
      execution: ExecutionWire {
        deadline_seconds: c.encode_deadline_seconds,
        queue_capacity: c.encode_queue_capacity,
        working_set_bytes: c.encode_working_set_bytes,
        concurrency: c.encode_concurrency,
      },
      limits: LimitsWire {
        input_bytes: c.max_input_bytes,
        output_bytes: c.max_output_bytes,
        dimension: c.max_dimension,
        pixels: c.max_pixels,
        decode_bytes: c.max_decode_bytes,
      },
    }
  }
}
impl Default for MediaConfig {
  fn default() -> Self {
    Self {
      webp_quality: 80.0,
      avif_quality: 80,
      avif_speed: 6,
      avif_threads: 1,
      encode_deadline_seconds: 30,
      encode_queue_capacity: 8,
      max_output_bytes: 64 * 1024 * 1024,
      auto_optimize_formats: if cfg!(feature = "jxl") {
        vec![AutoOptimizeImageFormat::Jxl, AutoOptimizeImageFormat::Webp]
      } else {
        vec![AutoOptimizeImageFormat::Webp]
      },
      max_input_bytes: 32 * 1024 * 1024,
      max_dimension: 8192,
      max_pixels: 16_777_216,
      max_decode_bytes: 128 * 1024 * 1024,
      encode_working_set_bytes: 512 * 1024 * 1024,
      encode_concurrency: 1,
    }
  }
}
impl MediaConfig {
  pub fn validate(&self) -> RecorderResult<()> {
    if !self.webp_quality.is_finite()
      || !(0.0..=100.0).contains(&self.webp_quality)
      || self.avif_quality > 100
      || !(1..=10).contains(&self.avif_speed)
      || self.avif_threads == 0
      || self.max_input_bytes == 0
      || self.max_dimension == 0
      || self.max_pixels == 0
      || self.max_decode_bytes == 0
      || self.encode_working_set_bytes == 0
      || self.encode_deadline_seconds == 0
      || self.encode_deadline_seconds > 300
      || self.max_output_bytes == 0
      || self.max_output_bytes > 64 * 1024 * 1024
      || self.encode_queue_capacity == 0
      || self.encode_queue_capacity > 1024
      || self.encode_working_set_bytes > u32::MAX as u64
      || self.encode_concurrency == 0
      || self.encode_concurrency > 64
    {
      return Err(RecorderError::InvalidConfiguration {
        message: "Invalid configuration at media: invalid image options/resource budget".into(),
      });
    }
    if self.auto_optimize_formats.contains(&AutoOptimizeImageFormat::Avif) {
      return Err(RecorderError::InvalidConfiguration {
        message: "Invalid configuration at media.auto_optimize_formats: AVIF is direct API only".into(),
      });
    }
    if self.auto_optimize_formats.contains(&AutoOptimizeImageFormat::Jxl) && !self.auto_optimize_formats.contains(&AutoOptimizeImageFormat::Webp) {
      return Err(RecorderError::InvalidConfiguration {
        message: "Invalid configuration at media.auto_optimize_formats: JXL requires WebP compatibility derivative".into(),
      });
    }
    if !cfg!(feature = "jxl") && self.auto_optimize_formats.contains(&AutoOptimizeImageFormat::Jxl) {
      return Err(RecorderError::InvalidConfiguration {
        message: "Invalid configuration at media.auto_optimize_formats: jxl feature is not enabled".into(),
      });
    }
    Ok(())
  }
}
