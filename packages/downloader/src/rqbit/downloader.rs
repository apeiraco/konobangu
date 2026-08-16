use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use librqbit::{AddTorrent, AddTorrentOptions, ManagedTorrent, Session, SessionOptions, api::TorrentIdOrHash};
use librqbit_core::Id20;
use snafu::ResultExt;
use tracing::instrument;
use util::errors::AnyhowResultExt;

use super::task::{RqbitCreation, RqbitHash, RqbitSelector, RqbitState, RqbitTask};
use crate::{
  DownloaderError,
  bittorrent::{
    downloader::TorrentDownloaderTrait,
    source::{HashTorrentSource, HashTorrentSourceTrait},
  },
  core::{DownloadIdSelector, DownloaderTrait},
  errors::RqbitSnafu,
};

#[derive(Debug)]
pub struct RqbitDownloaderCreation {
  pub save_path: String,
  pub subscriber_id: i32,
  pub downloader_id: i32,
}

impl RqbitDownloaderCreation {}

pub struct RqbitDownloader {
  pub save_path: String,
  pub subscriber_id: i32,
  pub downloader_id: i32,
  pub session: Arc<Session>,
}

impl RqbitDownloader {
  #[instrument(level = "debug")]
  pub async fn from_creation(creation: RqbitDownloaderCreation) -> Result<Arc<Self>, DownloaderError> {
    Self::from_creation_with_options(creation, SessionOptions::default()).await
  }

  pub async fn from_creation_with_options(creation: RqbitDownloaderCreation, session_opt: SessionOptions) -> Result<Arc<Self>, DownloaderError> {
    let session = Session::new_with_opts(creation.save_path.clone().into(), session_opt)
      .await
      .to_dyn_boxed()
      .context(RqbitSnafu {})?;
    Ok(Arc::new(Self {
      session,
      save_path: creation.save_path,
      subscriber_id: creation.subscriber_id,
      downloader_id: creation.downloader_id,
    }))
  }

  pub async fn add_torrent(&self, source: HashTorrentSource, opt: Option<AddTorrentOptions>) -> Result<RqbitHash, DownloaderError> {
    let hash = Id20::from_str(&source.hash_info() as &str).to_dyn_boxed().context(RqbitSnafu {})?;

    let source = match source {
      HashTorrentSource::TorrentFile(file) => AddTorrent::TorrentFileBytes(file.payload),
      HashTorrentSource::MagnetUrl(magnet) => AddTorrent::Url(magnet.url.into()),
    };
    let response = self.session.add_torrent(source, opt).await.to_dyn_boxed().context(RqbitSnafu {})?;

    let handle = response
      .into_handle()
      .ok_or_else(|| anyhow::anyhow!("failed to get handle of add torrent task"))
      .to_dyn_boxed()
      .context(RqbitSnafu {})?;

    handle.wait_until_initialized().await.to_dyn_boxed().context(RqbitSnafu {})?;

    Ok(hash)
  }

  fn query_torrent_impl(&self, hash: RqbitHash) -> Result<Arc<ManagedTorrent>, DownloaderError> {
    let torrent = self
      .session
      .get(TorrentIdOrHash::Hash(hash))
      .ok_or_else(|| anyhow::anyhow!("could not find torrent by hash {}", hash.as_string()))
      .to_dyn_boxed()
      .context(RqbitSnafu {})?;

    Ok(torrent)
  }

  pub fn query_torrent(&self, hash: RqbitHash) -> Result<RqbitTask, DownloaderError> {
    let torrent = self.query_torrent_impl(hash)?;

    let task = RqbitTask::from_query(torrent)?;

    Ok(task)
  }

  pub async fn pause_torrent(&self, hash: RqbitHash) -> Result<(), DownloaderError> {
    let t = self.query_torrent_impl(hash)?;
    self.session.pause(&t).await.to_dyn_boxed().context(RqbitSnafu {})?;

    Ok(())
  }

  pub async fn resume_torrent(&self, hash: RqbitHash) -> Result<(), DownloaderError> {
    let t = self.query_torrent_impl(hash)?;
    self.session.unpause(&t).await.to_dyn_boxed().context(RqbitSnafu {})?;

    Ok(())
  }

  pub async fn delete_torrent(&self, hash: RqbitHash) -> Result<(), DownloaderError> {
    self.delete_torrent_with_files(hash, true).await
  }

  pub async fn shutdown(&self) {
    self.session.stop().await;
  }

  pub async fn delete_torrent_with_files(&self, hash: RqbitHash, delete_files: bool) -> Result<(), DownloaderError> {
    if self.session.get(TorrentIdOrHash::Hash(hash)).is_none() {
      return Ok(());
    }
    self
      .session
      .delete(TorrentIdOrHash::Hash(hash), delete_files)
      .await
      .to_dyn_boxed()
      .context(RqbitSnafu {})?;

    Ok(())
  }
}

#[async_trait]
impl DownloaderTrait for RqbitDownloader {
  type State = RqbitState;
  type Id = RqbitHash;
  type Task = RqbitTask;
  type Creation = RqbitCreation;
  type Selector = RqbitSelector;

  #[instrument(level = "debug", skip(self))]
  async fn add_downloads(&self, creation: RqbitCreation) -> Result<Vec<<Self as DownloaderTrait>::Id>, DownloaderError> {
    let output = if creation.save_path.as_str().is_empty() {
      self.save_path.clone()
    } else {
      creation.save_path.to_string()
    };
    let mut sources = creation.sources;
    if sources.len() == 1 {
      let hash = self
        .add_torrent(
          sources.pop().unwrap(),
          Some(AddTorrentOptions {
            paused: false,
            output_folder: Some(output.clone()),
            ..Default::default()
          }),
        )
        .await?;
      Ok(vec![hash])
    } else {
      let tasks = sources
        .into_iter()
        .map(|s| {
          self.add_torrent(
            s,
            Some(AddTorrentOptions {
              paused: false,
              output_folder: Some(output.clone()),
              ..Default::default()
            }),
          )
        })
        .collect::<Vec<_>>();
      let results = futures::future::try_join_all(tasks).await?;
      Ok(results)
    }
  }

  async fn pause_downloads(&self, selector: <Self as DownloaderTrait>::Selector) -> Result<impl IntoIterator<Item = Self::Id>, DownloaderError> {
    <Self as TorrentDownloaderTrait>::pause_downloads(self, selector).await
  }

  async fn resume_downloads(&self, selector: <Self as DownloaderTrait>::Selector) -> Result<impl IntoIterator<Item = Self::Id>, DownloaderError> {
    <Self as TorrentDownloaderTrait>::resume_downloads(self, selector).await
  }

  async fn remove_downloads(&self, selector: <Self as DownloaderTrait>::Selector) -> Result<impl IntoIterator<Item = Self::Id>, DownloaderError> {
    <Self as TorrentDownloaderTrait>::remove_downloads(self, selector).await
  }

  #[instrument(level = "debug", skip(self))]
  async fn query_downloads(&self, selector: RqbitSelector) -> Result<Vec<<Self as DownloaderTrait>::Task>, DownloaderError> {
    let hashes = selector.into_iter();

    let tasks = hashes.map(|h| self.query_torrent(h)).collect::<Result<Vec<_>, DownloaderError>>()?;

    Ok(tasks)
  }
}

#[async_trait]
impl TorrentDownloaderTrait for RqbitDownloader {
  type IdSelector = DownloadIdSelector<Self::Task>;

  #[instrument(level = "debug", skip(self))]
  async fn pause_torrents(&self, selector: Self::IdSelector) -> Result<Self::IdSelector, DownloaderError> {
    let mut hashes: Vec<_> = selector.clone();

    if hashes.len() == 1 {
      self.pause_torrent(hashes.pop().unwrap()).await?;
    } else {
      futures::future::try_join_all(hashes.into_iter().map(|h| self.pause_torrent(h))).await?;
    }
    Ok(selector)
  }

  #[instrument(level = "debug", skip(self))]
  async fn resume_torrents(&self, selector: Self::IdSelector) -> Result<Self::IdSelector, DownloaderError> {
    let mut hashes: Vec<_> = selector.clone();

    if hashes.len() == 1 {
      self.resume_torrent(hashes.pop().unwrap()).await?;
    } else {
      futures::future::try_join_all(hashes.into_iter().map(|h| self.resume_torrent(h))).await?;
    }
    Ok(selector)
  }

  #[instrument(level = "debug", skip(self))]
  async fn remove_torrents(&self, selector: Self::IdSelector) -> Result<Self::IdSelector, DownloaderError> {
    let mut hashes: Vec<_> = selector.clone();

    if hashes.len() == 1 {
      self.delete_torrent(hashes.pop().unwrap()).await?;
    } else {
      futures::future::try_join_all(hashes.into_iter().map(|h| self.delete_torrent(h))).await?;
    }
    Ok(selector)
  }
}
