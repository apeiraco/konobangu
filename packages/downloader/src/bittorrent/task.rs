use std::{borrow::Cow, hash::Hash};

use quirks_path::{Path, PathBuf};

use crate::{
  bittorrent::source::HashTorrentSource,
  core::{DownloadCreationTrait, DownloadIdTrait, DownloadStateTrait, DownloadTaskTrait},
};

pub const TORRENT_TAG_NAME: &str = "konobangu";

pub trait TorrentHashTrait: DownloadIdTrait + Send + Hash {}

pub type SimpleTorrentHash = String;

impl DownloadIdTrait for SimpleTorrentHash {}

impl TorrentHashTrait for SimpleTorrentHash {}

pub trait TorrentStateTrait: DownloadStateTrait {}

pub trait TorrentTaskTrait: DownloadTaskTrait
where
  Self::State: TorrentStateTrait,
  Self::Id: TorrentHashTrait,
{
  fn hash_info(&self) -> Cow<'_, str>;

  fn name(&self) -> Cow<'_, str> {
    self.hash_info()
  }

  fn tags(&self) -> impl Iterator<Item = Cow<'_, str>>;

  fn category(&self) -> Option<Cow<'_, str>>;
}

pub trait TorrentCreationTrait: DownloadCreationTrait {
  fn save_path(&self) -> &Path;

  fn save_path_mut(&mut self) -> &mut PathBuf;

  fn sources_mut(&mut self) -> &mut Vec<HashTorrentSource>;
}
