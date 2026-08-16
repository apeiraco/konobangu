use recorder::media::MediaConfig;

pub fn config() -> MediaConfig {
  MediaConfig::default()
}

#[cfg(feature = "jxl")]
#[path = "jxl_decode.rs"]
mod decoder;
#[cfg(feature = "jxl")]
pub use decoder::decode;
