use std::{borrow::Cow, fmt::Debug, sync::Arc, time::Duration};

use librqbit::{ManagedTorrent, ManagedTorrentState, TorrentStats, TorrentStatsState};
use librqbit_core::Id20;
use quirks_path::{Path, PathBuf};

use crate::{
    DownloaderError,
    bittorrent::{
        source::HashTorrentSource,
        task::{TorrentCreationTrait, TorrentHashTrait, TorrentStateTrait, TorrentTaskTrait},
    },
    core::{
        DownloadCreationTrait, DownloadIdSelector, DownloadIdTrait, DownloadSimpleState,
        DownloadStateTrait, DownloadTaskTrait,
    },
};

pub type RqbitHash = Id20;

impl DownloadIdTrait for RqbitHash {}

impl TorrentHashTrait for RqbitHash {}

#[derive(Debug, Clone)]
pub struct RqbitState(Arc<TorrentStats>);

impl DownloadStateTrait for RqbitState {
    fn to_download_state(&self) -> DownloadSimpleState {
        match self.0.state {
            TorrentStatsState::Error => DownloadSimpleState::Error,
            TorrentStatsState::Paused => DownloadSimpleState::Paused,
            TorrentStatsState::Live => {
                if self.0.finished {
                    DownloadSimpleState::Completed
                } else {
                    DownloadSimpleState::Active
                }
            }
            TorrentStatsState::Initializing => DownloadSimpleState::Active,
        }
    }
}

impl TorrentStateTrait for RqbitState {}

impl From<Arc<TorrentStats>> for RqbitState {
    fn from(value: Arc<TorrentStats>) -> Self {
        Self(value)
    }
}

pub struct RqbitTask {
    pub hash_info: RqbitHash,
    pub torrent: Arc<ManagedTorrent>,
    pub state: RqbitState,
    pub stats: Arc<TorrentStats>,
}

impl RqbitTask {
    pub fn from_query(torrent: Arc<ManagedTorrent>) -> Result<Self, DownloaderError> {
        let hash = torrent.info_hash();
        let stats = Arc::new(torrent.stats());
        Ok(Self {
            hash_info: hash,
            state: stats.clone().into(),
            stats,
            torrent,
        })
    }
}

impl Debug for RqbitTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RqbitTask")
            .field("hash_info", &self.hash_info)
            .field("state", &self.id())
            .finish()
    }
}

impl DownloadTaskTrait for RqbitTask {
    type State = RqbitState;
    type Id = RqbitHash;

    fn id(&self) -> &Self::Id {
        &self.hash_info
    }

    fn into_id(self) -> Self::Id {
        self.hash_info
    }

    fn name(&self) -> Cow<'_, str> {
        self.torrent
            .metadata
            .load_full()
            .and_then(|m| m.info.name().map(|n| n.to_string()))
            .map(Cow::Owned)
            .unwrap_or_else(|| DownloadTaskTrait::name(self))
    }

    fn speed(&self) -> Option<u64> {
        self.stats
            .live
            .as_ref()
            .map(|s| s.download_speed.mbps)
            .and_then(|u| {
                let v = u * 1024f64 * 1024f64;
                if v.is_finite() && v > 0.0 && v < u64::MAX as f64 {
                    Some(v as u64)
                } else {
                    None
                }
            })
    }

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn dl_bytes(&self) -> Option<u64> {
        Some(self.stats.progress_bytes)
    }

    fn total_bytes(&self) -> Option<u64> {
        Some(self.stats.total_bytes)
    }

    fn et(&self) -> Option<Duration> {
        self.torrent.with_state(|l| match l {
            ManagedTorrentState::Live(l) => Some(Duration::from_millis(
                l.stats_snapshot().total_piece_download_ms,
            )),
            _ => None,
        })
    }

    fn eta(&self) -> Option<Duration> {
        self.torrent.with_state(|l| match l {
            ManagedTorrentState::Live(l) => l.down_speed_estimator().time_remaining(),
            _ => None,
        })
    }
}

impl TorrentTaskTrait for RqbitTask {
    fn hash_info(&self) -> Cow<'_, str> {
        Cow::Owned(self.hash_info.as_string())
    }

    fn tags(&self) -> impl Iterator<Item = Cow<'_, str>> {
        std::iter::empty()
    }

    fn category(&self) -> Option<Cow<'_, str>> {
        None
    }
}

#[derive(Debug, Clone, Default)]
pub struct RqbitCreation {
    pub save_path: PathBuf,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub sources: Vec<HashTorrentSource>,
}

impl DownloadCreationTrait for RqbitCreation {
    type Task = RqbitTask;
}

impl TorrentCreationTrait for RqbitCreation {
    fn save_path(&self) -> &Path {
        self.save_path.as_ref()
    }

    fn save_path_mut(&mut self) -> &mut PathBuf {
        &mut self.save_path
    }

    fn sources_mut(&mut self) -> &mut Vec<HashTorrentSource> {
        &mut self.sources
    }
}

pub type RqbitHashSelector = DownloadIdSelector<RqbitTask>;

pub type RqbitSelector = RqbitHashSelector;
