pub mod downloader;
pub mod task;

#[cfg(test)]
mod test;

pub use downloader::{QBittorrentDownloader, QBittorrentDownloaderCreation, QBittorrentSyncData};
pub use task::{
  QBittorrentComplexSelector, QBittorrentCreation, QBittorrentHash, QBittorrentHashSelector, QBittorrentSelector, QBittorrentState, QBittorrentTask,
};
