use std::{
    borrow::Cow,
    fmt::{Debug, Formatter},
};

use bytes::Bytes;
use librqbit::CloneToOwned;
use fetch::{bytes::fetch_bytes, client::HttpClientTrait};
use librqbit_core::{magnet::Magnet, torrent_metainfo, torrent_metainfo::TorrentMetaV1Owned};
use snafu::ResultExt;
use url::Url;
use util::errors::AnyhowResultExt;

use super::defs::MAGNET_SCHEMA;
use crate::errors::{DownloadFetchSnafu, DownloaderError, MagnetFormatSnafu, TorrentMetaSnafu};

pub trait HashTorrentSourceTrait: Sized {
    fn hash_info(&self) -> Cow<'_, str>;
}

pub struct MagnetUrlSource {
    pub magnet: Magnet,
    pub url: String,
}

impl MagnetUrlSource {
    pub fn from_url(url: String) -> Result<Self, DownloaderError> {
        let magnet = Magnet::parse(&url)
            .to_dyn_boxed()
            .context(MagnetFormatSnafu {
                message: url.clone(),
            })?;

        Ok(Self { magnet, url })
    }
}

impl HashTorrentSourceTrait for MagnetUrlSource {
    fn hash_info(&self) -> Cow<'_, str> {
        let hash_info = self
            .magnet
            .as_id32()
            .map(|s| s.as_string())
            .or_else(|| self.magnet.as_id20().map(|s| s.as_string()))
            .unwrap_or_else(|| unreachable!("hash of magnet must existed"));
        hash_info.into()
    }
}

impl Debug for MagnetUrlSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MagnetUrlSource")
            .field("url", &self.url)
            .finish()
    }
}

impl Clone for MagnetUrlSource {
    fn clone(&self) -> Self {
        Self {
            magnet: Magnet::parse(&self.url).unwrap(),
            url: self.url.clone(),
        }
    }
}

impl PartialEq for MagnetUrlSource {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl Eq for MagnetUrlSource {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TorrentUrlSource {
    pub url: String,
}

impl TorrentUrlSource {
    pub fn from_url(url: String) -> Result<Self, DownloaderError> {
        Ok(Self { url })
    }
}

#[derive(Clone)]
pub struct TorrentFileSource {
    pub url: Option<String>,
    pub payload: Bytes,
    pub meta: Box<TorrentMetaV1Owned>,
    pub filename: String,
}

impl TorrentFileSource {
    pub fn from_bytes(
        filename: String,
        bytes: Bytes,
        url: Option<String>,
    ) -> Result<Self, DownloaderError> {
        let meta = torrent_metainfo::torrent_from_bytes(bytes.as_ref())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            .with_context(|_| TorrentMetaSnafu {
                message: format!(
                    "filename = {}, url = {}",
                    filename,
                    url.as_deref().unwrap_or_default()
                ),
            })?
            .clone_to_owned(None);

        Ok(TorrentFileSource {
            url,
            payload: bytes,
            meta: Box::new(meta),
            filename,
        })
    }
    pub async fn from_url_and_http_client(
        client: &impl HttpClientTrait,
        url: String,
    ) -> Result<TorrentFileSource, DownloaderError> {
        let payload = fetch_bytes(client, &url)
            .await
            .boxed()
            .with_context(|_| DownloadFetchSnafu { url: url.clone() })?;

        let filename = Url::parse(&url)
            .boxed()
            .and_then(|s| {
                s.path_segments()
                    .and_then(|mut p| p.next_back())
                    .map(String::from)
                    .ok_or_else(|| anyhow::anyhow!("invalid url"))
                    .to_dyn_boxed()
            })
            .with_context(|_| DownloadFetchSnafu { url: url.clone() })?;

        Self::from_bytes(filename, payload, Some(url))
    }
}

impl HashTorrentSourceTrait for TorrentFileSource {
    fn hash_info(&self) -> Cow<'_, str> {
        self.meta.info_hash.as_string().into()
    }
}

impl Debug for TorrentFileSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TorrentFileSource")
            .field("hash", &self.meta.info_hash.as_string())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub enum UrlTorrentSource {
    MagnetUrl(MagnetUrlSource),
    TorrentUrl(TorrentUrlSource),
}

impl UrlTorrentSource {
    pub fn from_url(url: String) -> Result<Self, DownloaderError> {
        let url_ = Url::parse(&url)?;
        let source = if url_.scheme() == MAGNET_SCHEMA {
            Self::from_magnet_url(url)?
        } else {
            Self::from_torrent_url(url)?
        };
        Ok(source)
    }

    pub fn from_magnet_url(url: String) -> Result<Self, DownloaderError> {
        let magnet_source = MagnetUrlSource::from_url(url)?;
        Ok(Self::MagnetUrl(magnet_source))
    }

    pub fn from_torrent_url(url: String) -> Result<Self, DownloaderError> {
        let torrent_source = TorrentUrlSource::from_url(url)?;
        Ok(Self::TorrentUrl(torrent_source))
    }
}

#[derive(Debug, Clone)]
pub enum HashTorrentSource {
    MagnetUrl(MagnetUrlSource),
    TorrentFile(TorrentFileSource),
}

impl HashTorrentSource {
    pub async fn from_url_and_http_client(
        client: &impl HttpClientTrait,
        url: String,
    ) -> Result<Self, DownloaderError> {
        let url_ = Url::parse(&url)?;
        let source = if url_.scheme() == MAGNET_SCHEMA {
            Self::from_magnet_url(url)?
        } else {
            Self::from_torrent_url_and_http_client(client, url).await?
        };
        Ok(source)
    }

    pub fn from_magnet_url(url: String) -> Result<Self, DownloaderError> {
        let magnet_source = MagnetUrlSource::from_url(url)?;
        Ok(Self::MagnetUrl(magnet_source))
    }

    pub async fn from_torrent_url_and_http_client(
        client: &impl HttpClientTrait,
        url: String,
    ) -> Result<Self, DownloaderError> {
        let torrent_source = TorrentFileSource::from_url_and_http_client(client, url).await?;
        Ok(Self::TorrentFile(torrent_source))
    }
}

impl HashTorrentSourceTrait for HashTorrentSource {
    fn hash_info(&self) -> Cow<'_, str> {
        match self {
            HashTorrentSource::MagnetUrl(m) => m.hash_info(),
            HashTorrentSource::TorrentFile(t) => t.hash_info(),
        }
    }
}
