use std::fmt;

use async_stream::try_stream;
use axum::{body::Body, response::Response};
use axum_extra::{TypedHeader, headers::Range};
use bytes::Bytes;
use futures::StreamExt;
use headers_accept::Accept;
use http::{HeaderValue, Method, StatusCode, header};
use opendal::{Buffer, Metadata, Operator, Reader, Writer, layers::LoggingLayer};
use quirks_path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use super::StorageConfig;
use crate::{
  errors::{RecorderError, RecorderResult},
  utils::http::build_no_satisfiable_content_range,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageContentCategory {
  Image,
}

impl AsRef<str> for StorageContentCategory {
  fn as_ref(&self) -> &str {
    match self {
      Self::Image => "image",
    }
  }
}

pub enum StorageStoredUrl {
  RelativePath { path: String },
  Absolute { url: Url },
}

impl AsRef<str> for StorageStoredUrl {
  fn as_ref(&self) -> &str {
    match &self {
      Self::Absolute { url } => url.as_str(),
      Self::RelativePath { path } => path,
    }
  }
}

impl fmt::Display for StorageStoredUrl {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_ref())
  }
}

#[derive(Debug, Clone)]
pub struct StorageService {
  pub data_dir: String,
  pub operator: Operator,
}

impl StorageService {
  pub async fn from_config(config: StorageConfig) -> RecorderResult<Self> {
    Ok(Self {
      data_dir: config.data_dir.to_string(),
      operator: Self::get_operator(&config.data_dir)?,
    })
  }

  pub fn get_operator(data_dir: &str) -> Result<Operator, opendal::Error> {
    let op = if cfg!(test) {
      Operator::new(opendal::services::Memory::default())?.layer(LoggingLayer::default())
    } else {
      Operator::new(opendal::services::Fs::default().root(data_dir))?.layer(LoggingLayer::default())
    };

    Ok(op)
  }

  pub fn build_subscriber_path(&self, subscriber_id: i32, path: impl AsRef<Path>) -> PathBuf {
    let mut p = PathBuf::from("/subscribers");
    p.push(subscriber_id.to_string());
    p.push(path);
    p
  }

  #[cfg(any(test, feature = "test-utils"))]
  pub fn build_test_path(&self, path: impl AsRef<Path>) -> PathBuf {
    let mut p = PathBuf::from("/test");
    p.push(path);
    p
  }

  pub fn build_public_path(&self, path: impl AsRef<Path>) -> PathBuf {
    let mut p = PathBuf::from("/public");
    p.push(path);
    p
  }

  pub fn build_subscriber_object_path(&self, subscriber_id: i32, content_category: StorageContentCategory, bucket: &str, object_name: &str) -> PathBuf {
    self.build_subscriber_path(subscriber_id, [content_category.as_ref(), bucket, object_name].iter().collect::<PathBuf>())
  }

  pub fn build_public_object_path(&self, content_category: StorageContentCategory, bucket: &str, object_name: &str) -> PathBuf {
    self.build_public_path([content_category.as_ref(), bucket, object_name].iter().collect::<PathBuf>())
  }

  pub async fn write<P: Into<PathBuf> + Send>(&self, path: P, data: Bytes) -> Result<StorageStoredUrl, opendal::Error> {
    let operator = &self.operator;

    let path = path.into();
    if path.extension().is_some_and(|e| matches!(e, "jpg" | "jpeg" | "png" | "webp")) {
      self.invalidate_derivatives(path.as_str()).await?;
    }

    if let Some(dirname) = path.parent() {
      let dirname = dirname.join("/");
      operator.create_dir(dirname.as_str()).await?;
    }

    // Memory writes atomically replace one value; only the filesystem needs
    // a temporary object and rename.
    if operator.info().scheme() == "memory" {
      operator.write(path.as_str(), data).await?;
      return Ok(StorageStoredUrl::RelativePath { path: path.to_string() });
    }
    let temporary = format!("{}.{}.part", path.as_str(), Uuid::now_v7());
    let _cleanup = self
      .stage_bytes(temporary.clone(), data)
      .await
      .map_err(|_| opendal::Error::new(opendal::ErrorKind::Unexpected, "Atomic image temporary write failed"))?;
    let published = self.publish_temporary(&temporary, path.as_str()).await;
    if published.is_err() {
      let _ = operator.delete(&temporary).await;
    }
    published?;

    Ok(StorageStoredUrl::RelativePath { path: path.to_string() })
  }

  /// Encoding/downloads happen before taking the short fence lock. Publication
  /// uses an atomic rename while the execution token cannot be replaced.
  pub async fn write_for_task<P: Into<PathBuf> + Send>(&self, db: &sea_orm::DatabaseConnection, path: P, data: Bytes) -> RecorderResult<StorageStoredUrl> {
    use sea_orm::TransactionTrait;
    let path = path.into();
    let temporary = format!("{}.{}.part", path.as_str(), Uuid::now_v7());
    if let Some(parent) = path.parent() {
      self.operator.create_dir(parent.join("/").as_str()).await?;
    }
    let _cleanup = self.stage_bytes(temporary.clone(), data).await?;
    let published = async {
      let transaction = db.begin().await?;
      crate::task::execution::check(&transaction).await?;
      crate::media::derivative::lock_resource(&transaction, path.as_str()).await?;
      crate::task::execution::check(&transaction).await?;
      if path.extension().is_some_and(|e| matches!(e, "jpg" | "jpeg" | "png" | "webp")) {
        self.invalidate_derivatives(path.as_str()).await?;
      }
      if self.operator.info().scheme() == "memory" {
        self.operator.write(path.as_str(), self.operator.read(&temporary).await?).await?;
        self.operator.delete(&temporary).await?;
      } else {
        self.publish_temporary(&temporary, path.as_str()).await?;
      }
      crate::database::operation::commit_task_transaction(transaction).await?;
      Ok::<_, RecorderError>(StorageStoredUrl::RelativePath { path: path.to_string() })
    }
    .await;
    if published.is_err() {
      let _ = self.operator.delete(&temporary).await;
    }
    published
  }

  pub async fn stage_bytes(&self, path: String, data: Bytes) -> RecorderResult<TemporaryFile> {
    let guard = self.temporary_guard(path.clone());
    if self.operator.info().scheme() == "fs" {
      let file = std::path::PathBuf::from(self.operator.info().root()).join(path.trim_start_matches('/'));
      tokio::task::spawn_blocking(move || {
        use std::io::Write;
        if let Some(parent) = file.parent() {
          std::fs::create_dir_all(parent)?;
        }
        let mut output = std::fs::OpenOptions::new().create_new(true).write(true).open(file)?;
        output.write_all(&data)?;
        Ok::<_, std::io::Error>(guard)
      })
      .await
      .map_err(|_| crate::media::invalid_options("Image temporary writer failed"))?
      .map_err(Into::into)
    } else {
      self.operator.write(&path, data).await?;
      Ok(guard)
    }
  }
  pub async fn publish_manifest(&self, source: &str, data: Bytes) -> RecorderResult<()> {
    let path = crate::media::derivative::manifest_path(source);
    if data.len() as u64 > crate::media::derivative::MANIFEST_LIMIT {
      return Err(crate::media::invalid_options("Image manifest budget exceeded"));
    }
    if self.operator.info().scheme() == "fs" {
      use std::io::Write;
      let temporary = format!("{path}.{}.part", Uuid::now_v7());
      let _guard = self.temporary_guard(temporary.clone());
      let root = std::path::PathBuf::from(self.operator.info().root());
      let mut file = std::fs::OpenOptions::new().create_new(true).write(true).open(root.join(&temporary))?;
      file.write_all(&data)?;
      drop(file);
      self.publish_temporary(&temporary, &path).await?;
    } else {
      self.write(path, data).await?;
    }
    Ok(())
  }
  pub fn temporary_guard(&self, path: String) -> TemporaryFile {
    TemporaryFile {
      operator: self.operator.clone(),
      path,
    }
  }
  pub async fn publish_temporary(&self, temporary: &str, target: &str) -> Result<(), opendal::Error> {
    if self.operator.info().scheme() == "memory" {
      self.operator.write(target, self.operator.read(temporary).await?).await?;
      self.operator.delete(temporary).await
    } else if self.operator.info().scheme() == "fs" {
      // Do not yield between the final fence check and the local atomic rename.
      let root = std::path::PathBuf::from(self.operator.info().root());
      std::fs::rename(root.join(temporary.trim_start_matches('/')), root.join(target.trim_start_matches('/')))
        .map_err(|_| opendal::Error::new(opendal::ErrorKind::Unexpected, "Atomic image publication failed"))
    } else {
      self.operator.rename(temporary, target).await
    }
  }
  pub async fn exists<P: ToString + Send>(&self, path: P) -> Result<Option<StorageStoredUrl>, opendal::Error> {
    let operator = &self.operator;

    let path = path.to_string();

    if operator.exists(&path).await? {
      Ok(Some(StorageStoredUrl::RelativePath { path }))
    } else {
      Ok(None)
    }
  }

  pub async fn read(&self, path: impl AsRef<str>) -> Result<Buffer, opendal::Error> {
    let operator = &self.operator;

    let data = operator.read(path.as_ref()).await?;

    Ok(data)
  }

  pub async fn reader(&self, path: impl AsRef<str>) -> Result<Reader, opendal::Error> {
    let operator = &self.operator;

    let reader = operator.reader(path.as_ref()).await?;

    Ok(reader)
  }

  pub async fn writer(&self, path: impl AsRef<str>) -> Result<Writer, opendal::Error> {
    let operator = &self.operator;

    let writer = operator.writer(path.as_ref()).await?;

    Ok(writer)
  }

  pub async fn stat(&self, path: impl AsRef<str>) -> Result<Metadata, opendal::Error> {
    let operator = &self.operator;

    let metadata = operator.stat(path.as_ref()).await?;

    Ok(metadata)
  }

  #[cfg(test)]
  pub async fn list_public(&self) -> Result<Vec<opendal::Entry>, opendal::Error> {
    use futures::TryStreamExt;
    let lister = self.operator.lister_with("public/").recursive(true).await?;
    lister.try_collect().await
  }

  #[cfg(test)]
  pub async fn list_subscribers(&self) -> Result<Vec<opendal::Entry>, opendal::Error> {
    use futures::TryStreamExt;
    let lister = self.operator.lister_with("subscribers/").recursive(true).await?;
    lister.try_collect().await
  }

  pub async fn serve_optimized_image(&self, storage_path: impl AsRef<Path>, range: Option<TypedHeader<Range>>, accept: Accept) -> RecorderResult<Response> {
    let media = crate::media::MediaService::from_config(crate::media::MediaConfig::default()).await?;
    let mut headers = http::HeaderMap::new();
    if let Some(TypedHeader(range)) = range {
      use axum_extra::headers::HeaderMapExt;
      headers.typed_insert(range);
    }
    self
      .serve_image(&Method::GET, storage_path.as_ref().as_str(), &headers, Some(&accept), &media, false)
      .await
  }
  pub async fn serve_image(
    &self,
    method: &Method,
    source: &str,
    headers: &http::HeaderMap,
    accept: Option<&Accept>,
    media: &crate::media::MediaService,
    private: bool,
  ) -> RecorderResult<Response> {
    use crate::media::{AutoOptimizeImageFormat as F, negotiation};
    self.stat(source).await?;
    let mut candidates: Vec<(String, String, Option<String>)> = Vec::new();
    let manifest = self.load_derivatives(source).await;
    let plans = media.derivative_plans();
    for format in [F::Jxl, F::Webp, F::Avif] {
      if let Some(plan) = plans.iter().find(|plan| plan.format == format)
        && let Some(entry) = manifest.as_ref().and_then(|m| {
          m.entries
            .iter()
            .find(|e| e.format == format && crate::media::derivative::accepted_profile(&plan.options, &e.profile))
        })
      {
        candidates.push((entry.path.clone(), format.mime().into(), Some(format!("\"{}\"", entry.sha256))));
      }
    }
    candidates.push((source.into(), mime_guess::from_path(source).first_or_octet_stream().to_string(), None));
    let selected = if let Some(accept) = accept {
      negotiation::validate_accept(accept)?;
      let types = candidates.iter().map(|(_, mime, _)| (mime.as_str(), mime == "image/jxl")).collect::<Vec<_>>();
      negotiation::choose(accept, &types)
    } else {
      Some(candidates.len() - 1)
    };
    let mut response = if let Some(selected) = selected {
      let (path, _, etag) = &candidates[selected];
      self.serve_representation(method, path, headers, etag.as_deref()).await?
    } else {
      Response::builder().status(StatusCode::NOT_ACCEPTABLE).body(Body::empty())?
    };
    super::merge_vary(response.headers_mut(), "Accept");
    response.headers_mut().insert(
      header::CACHE_CONTROL,
      HeaderValue::from_static(if private { "private, no-cache" } else { "public, no-cache" }),
    );
    Ok(response)
  }
  pub async fn serve_file(&self, storage_path: impl AsRef<Path>, range: Option<TypedHeader<Range>>) -> RecorderResult<Response> {
    use axum_extra::headers::HeaderMapExt;
    let mut headers = http::HeaderMap::new();
    if let Some(TypedHeader(range)) = range {
      headers.typed_insert(range);
    }
    self.serve_representation(&Method::GET, storage_path.as_ref().as_str(), &headers, None).await
  }
  pub async fn serve_representation(&self, method: &Method, path: &str, request: &http::HeaderMap, strong_etag: Option<&str>) -> RecorderResult<Response> {
    use axum_extra::headers::{ETag, HeaderMapExt, IfNoneMatch, IfRange, LastModified};
    let metadata = self.stat(path).await?;
    let len = metadata.content_length();
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    // Precise metadata is a weak validator, never a proof of content identity.
    let weak =
      crate::media::derivative::fingerprint(&metadata).map(|fp| format!("W/\"{}\"", crate::media::derivative::hash(format!("{path}:{fp}").as_bytes())));
    let etag = strong_etag.or(weak.as_deref()).and_then(|v| v.parse::<ETag>().ok());
    let last_modified = metadata
      .last_modified()
      .and_then(|lm| HeaderValue::from_str(&lm.format_http_date()).ok())
      .and_then(|v| {
        let mut h = http::HeaderMap::new();
        h.insert(header::LAST_MODIFIED, v);
        h.typed_get::<LastModified>()
      });
    let mut common = http::HeaderMap::new();
    common.insert(header::CONTENT_TYPE, HeaderValue::from_str(mime.as_ref())?);
    common.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    if let Some(etag) = &etag {
      common.typed_insert(etag.clone());
    }
    if let Some(lm) = &last_modified {
      common.typed_insert(*lm);
    }
    if let Some(condition) = request.typed_get::<IfNoneMatch>()
      && (condition == IfNoneMatch::any() || etag.as_ref().is_some_and(|tag| !condition.precondition_passes(tag)))
    {
      let mut response = Response::new(super::revalidation_body());
      *response.status_mut() = StatusCode::NOT_MODIFIED;
      *response.headers_mut() = common;
      return Ok(response);
    }
    // Dates cannot prove a strong representation validator in this storage
    // contract. Parsing only entity tags also rejects malformed/weak matches.
    let if_range_matches = if !request.contains_key(header::IF_RANGE) {
      true
    } else if request.get_all(header::IF_RANGE).iter().count() != 1 {
      false
    } else {
      let mut tags = http::HeaderMap::new();
      tags.insert(header::ETAG, request[header::IF_RANGE].clone());
      tags.typed_get::<ETag>().is_some_and(|tag| !IfRange::etag(tag).is_modified(etag.as_ref(), None))
    };
    let ranges = if method == Method::GET && if_range_matches {
      super::ranges::normalize(request, len)
    } else {
      super::ranges::Ranges::Ignore
    };
    if let super::ranges::Ranges::Invalid = ranges {
      let mut response = Response::new(Body::from("Invalid Range header"));
      *response.status_mut() = StatusCode::BAD_REQUEST;
      *response.headers_mut() = common;
      return Ok(response);
    }
    if let super::ranges::Ranges::Bytes(ranges) = ranges {
      let ranges = ranges
        .into_iter()
        .map(|bounds| {
          let value = HeaderValue::from_str(&format!("bytes {}-{}/{len}", bounds.start(), bounds.end())).expect("numeric Content-Range");
          (bounds, value)
        })
        .collect::<Vec<_>>();
      if ranges.is_empty() {
        let mut response = Response::new(Body::empty());
        *response.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
        *response.headers_mut() = common;
        response.headers_mut().insert(header::CONTENT_RANGE, build_no_satisfiable_content_range(len));
        return Ok(response);
      }
      if ranges.len() == 1 {
        let (bounds, content_range) = ranges.into_iter().next().unwrap();
        let stream = self.reader(path).await?.into_bytes_stream(bounds.clone()).await?;
        common.insert(header::CONTENT_RANGE, content_range);
        common.insert(header::CONTENT_LENGTH, HeaderValue::from(range_length(&bounds)));
        let mut response = Response::new(Body::from_stream(stream));
        *response.status_mut() = StatusCode::PARTIAL_CONTENT;
        *response.headers_mut() = common;
        return Ok(response);
      }
      let boundary = Uuid::now_v7().to_string();
      let mut total = 0u64;
      let mut parts = Vec::new();
      for (bounds, range) in ranges {
        let part = format!("--{boundary}\r\nContent-Type: {mime}\r\nContent-Range: {}\r\n\r\n", range.to_str().unwrap());
        let Some(next) = total
          .checked_add(part.len() as u64)
          .and_then(|n| n.checked_add(range_length(&bounds)))
          .and_then(|n| n.checked_add(2))
        else {
          return self.full_representation(method, path, common, len).await;
        };
        total = next;
        parts.push((bounds, part));
      }
      let closing = format!("--{boundary}--\r\n");
      let Some(total) = total.checked_add(closing.len() as u64) else {
        return self.full_representation(method, path, common, len).await;
      };
      let reader = self.reader(path).await?;
      let stream = try_stream! {for (bounds,part) in parts {yield Bytes::from(part);let mut stream=reader.clone().into_bytes_stream(bounds).await?;while let Some(chunk)=stream.next().await{yield chunk?;}yield Bytes::from_static(b"\r\n");}yield Bytes::from(closing);};
      let stream = stream.map(|r: Result<Bytes, RecorderError>| r);
      common.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&format!("multipart/byteranges; boundary={boundary}"))?,
      );
      common.insert(header::CONTENT_LENGTH, HeaderValue::from(total));
      let mut response = Response::new(Body::from_stream(stream));
      *response.status_mut() = StatusCode::PARTIAL_CONTENT;
      *response.headers_mut() = common;
      return Ok(response);
    }
    self.full_representation(method, path, common, len).await
  }
  async fn full_representation(&self, method: &Method, path: &str, mut common: http::HeaderMap, len: u64) -> RecorderResult<Response> {
    common.insert(header::CONTENT_LENGTH, HeaderValue::from(len));
    let body = if method == Method::HEAD {
      Body::empty()
    } else {
      Body::from_stream(self.reader(path).await?.into_bytes_stream(..).await?)
    };
    let mut response = Response::new(body);
    *response.headers_mut() = common;
    Ok(response)
  }
}
fn range_length(bounds: &std::ops::RangeInclusive<u64>) -> u64 {
  // Normalization guarantees start <= end < representation length.
  bounds.end() - bounds.start() + 1
}

pub struct TemporaryFile {
  operator: Operator,
  path: String,
}
impl Drop for TemporaryFile {
  fn drop(&mut self) {
    if self.operator.info().scheme() == "fs" {
      let root = std::path::PathBuf::from(self.operator.info().root());
      let _ = std::fs::remove_file(root.join(self.path.trim_start_matches('/')));
    } else {
      let operator = self.operator.clone();
      let path = self.path.clone();
      if let Ok(runtime) = tokio::runtime::Handle::try_current() {
        runtime.spawn(async move {
          let _ = operator.delete(&path).await;
        });
      }
    }
  }
}
