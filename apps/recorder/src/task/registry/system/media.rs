use std::sync::Arc;

use quirks_path::Path;
use tracing::instrument;

use crate::{
  app::AppContextTrait,
  errors::RecorderResult,
  media::EncodeImageOptions,
  task::{AsyncTaskTrait, register_system_task_type},
};

register_system_task_type! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct OptimizeImageTask {
        pub source_path: String,
        pub target_path: String,
        pub format_options: EncodeImageOptions,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[builder(default = None)]
        pub derivation_version: Option<u8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[builder(default = None)]
        pub source_fingerprint: Option<String>,
    }
}

#[async_trait::async_trait]
impl AsyncTaskTrait for OptimizeImageTask {
  #[instrument(err, skip(ctx))]
  async fn run_async(self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
    let storage = ctx.storage();

    let source_path = Path::new(&self.source_path);

    let media_service = ctx.media();

    use sea_orm::TransactionTrait;

    use crate::media::derivative::{self, DerivativeEntry, DerivativeManifest};
    if !derivative::validate_source(&self.source_path) {
      return Err(crate::media::invalid_options("Invalid image source namespace"));
    }
    if let Some(private) = self.source_path.trim_start_matches('/').strip_prefix("subscribers/")
      && (private.split('/').next().and_then(|id| id.parse::<i32>().ok()) != self.subscriber_id || self.subscriber_id.is_none())
    {
      return Err(crate::auth::AuthError::PermissionError.into());
    }
    let before = storage.stat(&self.source_path).await?;
    if before.content_length() > media_service.config.max_input_bytes {
      return Err(crate::media::invalid_options("Image input byte budget exceeded"));
    }
    let fingerprint = derivative::fingerprint(&before).ok_or_else(|| crate::media::invalid_options("Image source has no reliable fingerprint"))?;
    if self.source_fingerprint.as_ref().is_some_and(|expected| expected != &fingerprint) {
      return Err(crate::media::invalid_options("Image source version changed"));
    }
    let image_data = storage
      .operator
      .read_with(&self.source_path)
      .range(0..before.content_length())
      .await?
      .to_bytes();
    let source_hash = derivative::hash(&image_data);
    let format = match &self.format_options {
      EncodeImageOptions::Webp(_) => crate::media::AutoOptimizeImageFormat::Webp,
      EncodeImageOptions::Avif(_) => crate::media::AutoOptimizeImageFormat::Avif,
      EncodeImageOptions::Jxl(_) => crate::media::AutoOptimizeImageFormat::Jxl,
    };
    let profile = derivative::profile(&self.format_options);
    let data = match self.format_options {
      EncodeImageOptions::Webp(options) => {
        if self.derivation_version.is_some() {
          media_service.optimize_cover_to_webp(image_data, Some(options)).await?
        } else {
          media_service.optimize_image_to_webp(source_path, image_data, Some(options)).await?
        }
      }
      EncodeImageOptions::Avif(options) => media_service.optimize_image_to_avif(source_path, image_data, Some(options)).await?,
      EncodeImageOptions::Jxl(options) => {
        if self.derivation_version.is_some() {
          media_service.optimize_cover_to_jxl(image_data, Some(options)).await?
        } else {
          media_service.optimize_image_to_jxl(source_path, image_data, Some(options)).await?
        }
      }
    };
    if self.derivation_version.is_none() {
      // Legacy jobs keep their direct URL; only new jobs publish negotiated
      // profiles.
      if Path::new(&self.target_path) != source_path.with_extension(format.extension()) {
        return Err(crate::media::invalid_options("Invalid legacy image target"));
      }
      let temporary = format!("{}.{}.part", self.target_path, uuid::Uuid::now_v7());
      let _cleanup = storage.stage_bytes(temporary.clone(), data).await?;
      let transaction = ctx.task().queue_database()?.begin().await?;
      crate::task::execution::check(&transaction).await?;
      derivative::lock_resource(&transaction, &self.source_path).await?;
      if derivative::fingerprint(&storage.stat(&self.source_path).await?) != Some(fingerprint) {
        return Err(crate::media::invalid_options("Image source version changed"));
      }
      // Recheck the already locked task row after any resource-lock wait.
      crate::task::execution::check(&transaction).await?;
      storage.publish_temporary(&temporary, &self.target_path).await?;
      crate::database::operation::commit_task_transaction(transaction).await?;
      return Ok(());
    }
    if !matches!(self.derivation_version, Some(1) | Some(derivative::VERSION) | Some(3)) {
      return Err(crate::media::invalid_options("Unsupported image derivation version"));
    }
    let content_hash = derivative::hash(&data);
    let target = derivative::derivative_path(&self.source_path, &profile, &source_hash, &content_hash, format);
    let temporary = format!("{target}.{}.part", uuid::Uuid::now_v7());
    let _cleanup = storage.stage_bytes(temporary.clone(), data.clone()).await?;
    let transaction = ctx.task().queue_database()?.begin().await?;
    crate::task::execution::check(&transaction).await?;
    derivative::lock_resource(&transaction, &self.source_path).await?;
    if derivative::fingerprint(&storage.stat(&self.source_path).await?) != Some(fingerprint.clone()) {
      return Err(crate::media::invalid_options("Image source version changed"));
    }
    let mut manifest = storage.load_derivatives(&self.source_path).await.unwrap_or(DerivativeManifest {
      version: derivative::VERSION,
      source_fingerprint: fingerprint,
      source_sha256: source_hash,
      entries: Vec::new(),
    });
    crate::task::execution::check(&transaction).await?;
    storage.publish_temporary(&temporary, &target).await?;
    manifest.entries.retain(|entry| entry.format != format);
    manifest.entries.push(DerivativeEntry {
      profile,
      format,
      path: target,
      length: data.len() as u64,
      sha256: content_hash,
    });
    storage
      .publish_manifest(&self.source_path, bytes::Bytes::from(serde_json::to_vec(&manifest)?))
      .await?;
    crate::database::operation::commit_task_transaction(transaction).await?;

    Ok(())
  }
}
