//! One profile and path contract shared by collection, workers and delivery.
use sea_orm::TransactionTrait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
  errors::RecorderResult,
  media::{AutoOptimizeImageFormat, EncodeAvifOptions, EncodeImageOptions, EncodeJxlOptions, EncodeWebpOptions, MediaService},
  storage::StorageService,
};
pub const VERSION: u8 = 2;
pub const MANIFEST_LIMIT: u64 = 64 * 1024;
#[derive(Clone, Debug)]
pub struct DerivativePlan {
  pub format: AutoOptimizeImageFormat,
  pub options: EncodeImageOptions,
  pub profile: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DerivativeEntry {
  pub profile: String,
  pub format: AutoOptimizeImageFormat,
  pub path: String,
  pub length: u64,
  pub sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DerivativeManifest {
  pub version: u8,
  pub source_fingerprint: String,
  pub source_sha256: String,
  pub entries: Vec<DerivativeEntry>,
}
pub fn hash(bytes: &[u8]) -> String {
  format!("{:x}", Sha256::digest(bytes))
}
pub fn manifest_path(source: &str) -> String {
  format!("{}.derivatives.json", source.trim_start_matches('/'))
}
pub fn derivative_path(source: &str, profile: &str, source_hash: &str, content_hash: &str, format: AutoOptimizeImageFormat) -> String {
  let source = source.trim_start_matches('/');
  format!("{source}.derived/v{VERSION}-{profile}-{source_hash}-{content_hash}.{}", format.extension())
}
pub fn profile(options: &EncodeImageOptions) -> String {
  let policy = match options {
    EncodeImageOptions::Jxl(_) => "jpxl-0.3.0-1e2004aa0672259279cc24a52d4b8e264f675b77-balanced77-threads1-cover1600-srgb-white-lanczos3-preset1",
    EncodeImageOptions::Webp(_) => "webp-0.3.1-libwebp-sys-0.9.6-q80-cover1600-srgb-white-lanczos3-preset1",
    EncodeImageOptions::Avif(_) => "image-0.25.10-avif-direct-v1",
  };
  policy_hash(policy, options)
}
fn policy_hash(policy: &str, options: &EncodeImageOptions) -> String {
  hash(format!("{policy}:{}", serde_json::to_string(options).expect("finite validated profile options")).as_bytes())
}
/// Qualified legacy profiles stay readable while new jobs target the new
/// recipe. Their original serialized options are part of the identity, never
/// translated.
pub fn accepted_profile(options: &EncodeImageOptions, candidate: &str) -> bool {
  if candidate == profile(options) {
    return true;
  }
  match options {
    EncodeImageOptions::Jxl(_) => [
      "cover-jxl-v1-libjxl-0.12.0-1600-premultiplied-lanczos3-alpha0-ac-dc0",
      "builtin-jxl-sys-0.1.12-libjxl-0.12.0-cover1600-premultiplied-lanczos3-alpha0-ac-dc0-v3",
    ]
    .iter()
    .any(|policy| candidate == hash(format!("{policy}:{{\"mime_type\":\"image/jxl\",\"distance\":2.8,\"effort\":7}}").as_bytes())),
    EncodeImageOptions::Webp(_) => [
      "webp-0.3.1-cover1600-premultiplied-lanczos3-v2",
      "isolated-webp-0.3.1-cover1600-premultiplied-lanczos3-v3",
    ]
    .iter()
    .any(|policy| candidate == policy_hash(policy, options)),
    EncodeImageOptions::Avif(_) => false,
  }
}

pub fn validate_source(source: &str) -> bool {
  let path = source.trim_start_matches('/');
  (path.starts_with("public/") || path.starts_with("subscribers/")) && path.split('/').all(|p| !p.is_empty() && p != "." && p != ".." && !p.contains('\\'))
}
pub fn fingerprint(metadata: &opendal::Metadata) -> Option<String> {
  metadata
    .last_modified()
    .map(|t| format!("{}:{}", metadata.content_length(), t.into_inner().as_nanosecond()))
    .or_else(|| metadata.etag().map(|etag| format!("{}:{etag}", metadata.content_length())))
}
impl MediaService {
  pub fn derivative_plans(&self) -> Vec<DerivativePlan> {
    self
      .config
      .auto_optimize_formats
      .iter()
      .map(|format| {
        let options = match format {
          AutoOptimizeImageFormat::Webp => EncodeImageOptions::Webp(EncodeWebpOptions { quality: Some(80.0) }),
          AutoOptimizeImageFormat::Avif => EncodeImageOptions::Avif(EncodeAvifOptions {
            quality: Some(self.config.avif_quality),
            speed: Some(self.config.avif_speed),
            threads: Some(self.config.avif_threads),
          }),
          AutoOptimizeImageFormat::Jxl => EncodeImageOptions::Jxl(EncodeJxlOptions::default()),
        };
        DerivativePlan {
          format: *format,
          profile: profile(&options),
          options,
        }
      })
      .collect()
  }
}
impl StorageService {
  pub async fn load_derivatives(&self, source: &str) -> Option<DerivativeManifest> {
    if !validate_source(source) {
      return None;
    }
    let expected = fingerprint(&self.stat(source).await.ok()?)?;
    let path = manifest_path(source);
    let meta = self.stat(&path).await.ok()?;
    if meta.content_length() > MANIFEST_LIMIT {
      return None;
    }
    let bytes = self.operator.read_with(&path).range(0..meta.content_length()).await.ok()?.to_bytes();
    if bytes.len() as u64 > MANIFEST_LIMIT {
      return None;
    }
    let manifest: DerivativeManifest = serde_json::from_slice(&bytes).ok()?;
    if manifest.version != VERSION || manifest.source_fingerprint != expected || !is_hash(&manifest.source_sha256) || manifest.entries.len() > 3 {
      return None;
    }
    let mut seen = std::collections::HashSet::new();
    for entry in &manifest.entries {
      if !is_hash(&entry.profile)
        || !is_hash(&entry.sha256)
        || !seen.insert(entry.format)
        || entry.path != derivative_path(source, &entry.profile, &manifest.source_sha256, &entry.sha256, entry.format)
        || self.stat(&entry.path).await.ok()?.content_length() != entry.length
      {
        return None;
      }
    }
    Some(manifest)
  }
  pub async fn missing_derivatives(&self, source: &str, media: &MediaService) -> Vec<DerivativePlan> {
    let manifest = self.load_derivatives(source).await;
    media
      .derivative_plans()
      .into_iter()
      .filter(|plan| {
        !manifest
          .as_ref()
          .is_some_and(|m| m.entries.iter().any(|e| e.profile == plan.profile && e.format == plan.format))
      })
      .collect()
  }
  pub async fn invalidate_derivatives(&self, source: &str) -> Result<(), opendal::Error> {
    if self.operator.info().scheme() == "fs" {
      let path = std::path::PathBuf::from(self.operator.info().root()).join(manifest_path(source));
      return match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(opendal::Error::new(opendal::ErrorKind::Unexpected, "Image manifest invalidation failed")),
      };
    }
    self.operator.delete(&manifest_path(source)).await
  }
}
fn is_hash(value: &str) -> bool {
  value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
pub async fn lock_resource<C: sea_orm::ConnectionTrait>(db: &C, source: &str) -> Result<(), sea_orm::DbErr> {
  use sea_orm::{DbBackend, Statement};
  let digest = Sha256::digest(source.trim_start_matches('/').as_bytes());
  let key = i64::from_be_bytes(digest[..8].try_into().unwrap());
  db.query_one_raw(Statement::from_sql_and_values(
    DbBackend::Postgres,
    "SELECT pg_advisory_xact_lock($1)",
    [(key ^ 0x4d45444941000000).into()],
  ))
  .await?;
  Ok(())
}
pub async fn enqueue_missing(ctx: &dyn crate::app::AppContextTrait, source: &str) -> RecorderResult<usize> {
  let source = source.trim_start_matches('/');
  if !validate_source(source) {
    return Err(super::service::invalid_options("Invalid image source namespace"));
  }
  let storage = ctx.storage();
  let media = ctx.media();
  let fingerprint = fingerprint(&storage.stat(source).await?).ok_or_else(|| super::service::invalid_options("Image source has no reliable fingerprint"))?;
  let transaction = ctx.task().queue_database()?.begin().await?;
  crate::task::execution::check(&transaction).await?;
  lock_resource(&transaction, source).await?;
  if self::fingerprint(&storage.stat(source).await?) != Some(fingerprint.clone()) {
    return Err(super::service::invalid_options("Image source version changed"));
  }
  // A worker may have completed while this caller waited for the resource.
  let plans = storage.missing_derivatives(source, media).await;
  crate::task::execution::check(&transaction).await?;
  let mut created = 0;
  for plan in plans {
    use sea_orm::{ConnectionTrait, DbBackend, Statement};
    let options = serde_json::to_value(&plan.options)?;
    let pending = transaction
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT id FROM task_runs WHERE task_type='optimize_image' AND status IN ('Pending','Scheduled','Running') AND cancel_requested_at IS NULL AND \
         archived_at IS NULL AND payload->>'source_path'=$1 AND payload->>'source_fingerprint'=$2 AND payload->'format_options'=$3 LIMIT 1",
        [source.to_string().into(), fingerprint.clone().into(), options.into()],
      ))
      .await?;
    if pending.is_some() {
      continue;
    }
    let mut task = crate::task::OptimizeImageTask::builder()
      .source_path(source.into())
      .target_path(format!("{source}.derived/{}", plan.profile))
      .format_options(plan.options)
      .derivation_version(Some(3))
      .source_fingerprint(Some(fingerprint.clone()))
      .build();
    if let Some(id) = source
      .trim_start_matches('/')
      .strip_prefix("subscribers/")
      .and_then(|s| s.split('/').next())
      .and_then(|s| s.parse::<i32>().ok())
    {
      task.subscriber_id = Some(id);
    }
    crate::task::operation::enqueue(&transaction, "system_task", &crate::task::SystemTask::from(task)).await?;
    created += 1;
  }
  crate::database::operation::commit_task_transaction(transaction).await?;

  Ok(created)
}
