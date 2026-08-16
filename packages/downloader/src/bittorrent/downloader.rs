use async_trait::async_trait;

use crate::{
  DownloaderError,
  bittorrent::task::{TorrentCreationTrait, TorrentHashTrait, TorrentStateTrait, TorrentTaskTrait},
  core::{DownloadIdSelectorTrait, DownloadSelectorTrait, DownloadTaskTrait, DownloaderTrait},
};

#[async_trait]
pub trait TorrentDownloaderTrait: DownloaderTrait
where
  Self::State: TorrentStateTrait,
  Self::Id: TorrentHashTrait,
  Self::Task: TorrentTaskTrait<State = Self::State, Id = Self::Id>,
  Self::Creation: TorrentCreationTrait<Task = Self::Task>,
  Self::Selector: DownloadSelectorTrait<Task = Self::Task, Id = Self::Id>,
{
  type IdSelector: DownloadIdSelectorTrait<Task = Self::Task, Id = Self::Id>;

  async fn pause_downloads(&self, selector: <Self as DownloaderTrait>::Selector) -> Result<Self::IdSelector, DownloaderError> {
    let hashes = <Self as TorrentDownloaderTrait>::query_torrent_hashes(self, selector).await?;
    self.pause_torrents(hashes).await
  }

  async fn resume_downloads(&self, selector: <Self as DownloaderTrait>::Selector) -> Result<Self::IdSelector, DownloaderError> {
    let hashes = <Self as TorrentDownloaderTrait>::query_torrent_hashes(self, selector).await?;
    self.resume_torrents(hashes).await
  }
  async fn remove_downloads(&self, selector: <Self as DownloaderTrait>::Selector) -> Result<Self::IdSelector, DownloaderError> {
    let hashes = <Self as TorrentDownloaderTrait>::query_torrent_hashes(self, selector).await?;
    self.remove_torrents(hashes).await
  }

  async fn query_torrent_hashes(&self, selector: <Self as DownloaderTrait>::Selector) -> Result<Self::IdSelector, DownloaderError> {
    let hashes = match selector.try_into_ids_only() {
      Ok(hashes) => Self::IdSelector::from_iter(hashes),
      Err(selector) => {
        let tasks = self.query_downloads(selector).await?;

        Self::IdSelector::from_iter(tasks.into_iter().map(|s| s.into_id()))
      }
    };
    Ok(hashes)
  }

  async fn pause_torrents(&self, hashes: Self::IdSelector) -> Result<Self::IdSelector, DownloaderError>;

  async fn resume_torrents(&self, hashes: Self::IdSelector) -> Result<Self::IdSelector, DownloaderError>;

  async fn remove_torrents(&self, hashes: Self::IdSelector) -> Result<Self::IdSelector, DownloaderError>;
}
