use std::{borrow::Cow, fmt};

use async_stream::try_stream;
use axum::{body::Body, response::Response};
use axum_extra::{TypedHeader, headers::Range};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use headers_accept::Accept;
use http::{HeaderValue, StatusCode, header};
use opendal::{Buffer, Metadata, Operator, Reader, Writer, layers::LoggingLayer};
use quirks_path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;
use uuid::Uuid;

use super::StorageConfig;
use crate::{
    errors::{RecorderError, RecorderResult},
    utils::http::{bound_range_to_content_range, build_no_satisfiable_content_range},
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
            Operator::new(opendal::services::Memory::default())?
                .layer(LoggingLayer::default())
                .finish()
        } else {
            Operator::new(opendal::services::Fs::default().root(data_dir))?
                .layer(LoggingLayer::default())
                .finish()
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

    pub fn build_subscriber_object_path(
        &self,
        subscriber_id: i32,
        content_category: StorageContentCategory,
        bucket: &str,
        object_name: &str,
    ) -> PathBuf {
        self.build_subscriber_path(
            subscriber_id,
            [content_category.as_ref(), bucket, object_name]
                .iter()
                .collect::<PathBuf>(),
        )
    }

    pub fn build_public_object_path(
        &self,
        content_category: StorageContentCategory,
        bucket: &str,
        object_name: &str,
    ) -> PathBuf {
        self.build_public_path(
            [content_category.as_ref(), bucket, object_name]
                .iter()
                .collect::<PathBuf>(),
        )
    }

    pub async fn write<P: Into<PathBuf> + Send>(
        &self,
        path: P,
        data: Bytes,
    ) -> Result<StorageStoredUrl, opendal::Error> {
        let operator = &self.operator;

        let path = path.into();

        if let Some(dirname) = path.parent() {
            let dirname = dirname.join("/");
            operator.create_dir(dirname.as_str()).await?;
        }

        operator.write(path.as_str(), data).await?;

        Ok(StorageStoredUrl::RelativePath {
            path: path.to_string(),
        })
    }

    pub async fn exists<P: ToString + Send>(
        &self,
        path: P,
    ) -> Result<Option<StorageStoredUrl>, opendal::Error> {
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
        let lister = self
            .operator
            .lister_with("subscribers/")
            .recursive(true)
            .await?;
        lister.try_collect().await
    }

    #[instrument(skip_all, err, fields(storage_path = %storage_path.as_ref(), range = ?range, accept = accept.to_string()))]
    pub async fn serve_optimized_image(
        &self,
        storage_path: impl AsRef<Path>,
        range: Option<TypedHeader<Range>>,
        accept: Accept,
    ) -> RecorderResult<Response> {
        let storage_path = Path::new(storage_path.as_ref());
        for mime_type in accept.media_types() {
            let accpetable_path = match mime_type.subty().as_str() {
                "webp" => Some(storage_path.with_extension("webp")),
                "avif" => Some(storage_path.with_extension("avif")),
                "jxl" => Some(storage_path.with_extension("jxl")),
                _ => None,
            };
            if let Some(accpetable_path) = accpetable_path
                && self.exists(&accpetable_path).await?.is_some()
                && self.stat(&accpetable_path).await?.is_file()
            {
                return self.serve_file(accpetable_path, range).await;
            }
        }

        self.serve_file(storage_path, range).await
    }

    #[instrument(skip_all, err, fields(storage_path = %storage_path.as_ref(), range = ?range))]
    pub async fn serve_file(
        &self,
        storage_path: impl AsRef<str>,
        range: Option<TypedHeader<Range>>,
    ) -> RecorderResult<Response> {
        let metadata = self
            .stat(&storage_path)
            .await
            .map_err(|_| RecorderError::from_status(StatusCode::NOT_FOUND))?;

        if !metadata.is_file() {
            return Err(RecorderError::from_status(StatusCode::NOT_FOUND));
        }

        let mime_type = mime_guess::from_path(storage_path.as_ref()).first_or_octet_stream();

        let content_type = HeaderValue::from_str(mime_type.as_ref())?;
        let etag = metadata.etag().map(Cow::Borrowed).or_else(|| {
            let len = metadata.content_length();
            let lm = metadata.last_modified()?.into_inner().as_second();
            Some(Cow::Owned(format!("\"{lm:x}-{len:x}\"")))
        });
        let last_modified = metadata.last_modified().map(|lm| lm.format_http_date());

        let response = if let Some(TypedHeader(range)) = range {
            let ranges = range
                .satisfiable_ranges(metadata.content_length())
                .map(|r| -> Option<(_, _)> {
                    let a = bound_range_to_content_range(&r, metadata.content_length())?;
                    Some((r, a))
                })
                .collect::<Option<Vec<_>>>();

            if let Some(mut ranges) = ranges {
                if ranges.len() > 1 {
                    let boundary = Uuid::now_v7().to_string();
                    let reader = self.reader(storage_path.as_ref()).await?;
                    let stream: impl Stream<Item = Result<Bytes, RecorderError>> = {
                        let boundary = boundary.clone();
                        try_stream! {
                            for (r, content_range) in ranges {
                                let part_header = format!("--{boundary}\r\nContent-Type: {}\r\nContent-Range: {}\r\n\r\n",
                                    mime_type.as_ref(),
                                    content_range.clone().to_str().unwrap(),
                                );
                                yield part_header.into();
                                let mut part_stream = reader.clone().into_bytes_stream(r).await?;
                                while let Some(chunk) = part_stream.next().await {
                                    yield chunk?;
                                }
                                yield "\r\n".into();
                            }
                            yield format!("--{boundary}--").into();
                        }
                    };
                    let body = Body::from_stream(stream);

                    let mut builder = Response::builder()
                        .status(StatusCode::PARTIAL_CONTENT)
                        .header(
                            header::CONTENT_TYPE,
                            HeaderValue::from_str(
                                format!("multipart/byteranges; boundary={boundary}").as_str(),
                            )
                            .unwrap(),
                        );

                    if let Some(etag) = etag {
                        builder = builder.header(header::ETAG, etag.to_string());
                    }

                    if let Some(last_modified) = last_modified {
                        builder = builder.header(header::LAST_MODIFIED, last_modified);
                    }

                    builder.body(body)?
                } else if let Some((r, content_range)) = ranges.pop() {
                    let reader = self.reader(storage_path.as_ref()).await?;
                    let stream = reader.into_bytes_stream(r).await?;

                    let mut builder = Response::builder()
                        .status(StatusCode::PARTIAL_CONTENT)
                        .header(header::CONTENT_TYPE, content_type.clone())
                        .header(header::CONTENT_RANGE, content_range);

                    if let Some(etag) = metadata.etag() {
                        builder = builder.header(header::ETAG, etag);
                    }
                    if let Some(last_modified) = last_modified {
                        builder = builder.header(header::LAST_MODIFIED, last_modified);
                    }

                    builder.body(Body::from_stream(stream))?
                } else {
                    unreachable!("ranges length should be greater than 0")
                }
            } else {
                Response::builder()
                    .status(StatusCode::RANGE_NOT_SATISFIABLE)
                    .header(header::CONTENT_TYPE, content_type)
                    .header(
                        header::CONTENT_RANGE,
                        build_no_satisfiable_content_range(metadata.content_length()),
                    )
                    .body(Body::empty())?
            }
        } else {
            let reader = self.reader(storage_path.as_ref()).await?;
            let stream = reader.into_bytes_stream(..).await?;

            let mut builder = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type);

            if let Some(etag) = etag {
                builder = builder.header(header::ETAG, etag.to_string());
            }

            if let Some(last_modified) = last_modified {
                builder = builder.header(header::LAST_MODIFIED, last_modified);
            }

            builder.body(Body::from_stream(stream))?
        };

        Ok(response)
    }
}
