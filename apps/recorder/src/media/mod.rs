mod config;
pub mod executor;
#[cfg(feature = "jxl")]
mod jxl;
mod service;

pub use config::{AutoOptimizeImageFormat, EncodeAvifOptions, EncodeImageOptions, EncodeJxlOptions, EncodeWebpOptions, MediaConfig};
pub use service::MediaService;

pub mod derivative;
pub use service::{decode, invalid_options, normalize_cover};

pub mod negotiation;

pub mod smoke;
