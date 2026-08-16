use crate::{
  errors::RecorderResult,
  media::{MediaConfig, MediaService},
};

pub async fn build_testing_media_service() -> RecorderResult<MediaService> {
  MediaService::from_config(MediaConfig::default()).await
}
