use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::{Arc, Weak},
    time::Duration,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use merge_struct::merge;
use qbit_rs::{
    Qbit,
    model::{
        AddTorrentArg, Category, Credential, GetTorrentListArg, NonEmptyStr, Sep, SyncData,
        Torrent as QbitTorrent, TorrentFile, TorrentSource,
    },
};
use quirks_path::PathBuf;
use snafu::{OptionExt, whatever};
use tokio::{
    sync::{RwLock, watch},
    time::sleep,
};
use tracing::instrument;
use url::Url;

use super::QBittorrentHashSelector;
use crate::{
    DownloaderError,
    bittorrent::{
        downloader::TorrentDownloaderTrait,
        source::{HashTorrentSource, HashTorrentSourceTrait, MagnetUrlSource, TorrentFileSource},
        task::TORRENT_TAG_NAME,
    },
    core::DownloaderTrait,
    qbit::task::{
        QBittorrentCreation, QBittorrentHash, QBittorrentSelector, QBittorrentState,
        QBittorrentTask,
    },
    utils::path_equals_as_file_url,
};

#[derive(Debug)]
pub struct QBittorrentDownloaderCreation {
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub save_path: String,
    pub subscriber_id: i32,
    pub downloader_id: i32,
    pub wait_sync_timeout: Option<Duration>,
}

#[derive(Default)]
pub struct QBittorrentSyncData {
    pub torrents: HashMap<String, QbitTorrent>,
    pub categories: HashMap<String, Category>,
    pub tags: HashSet<String>,
    pub trackers: HashMap<String, Vec<String>>,
    pub server_state: HashMap<String, serde_value::Value>,
    pub rid: i64,
}

impl QBittorrentSyncData {
    pub fn patch(&mut self, data: SyncData) {
        self.rid = data.rid;
        if data.full_update.is_some_and(|s| s) {
            self.torrents.clear();
            self.categories.clear();
            self.tags.clear();
            self.trackers.clear();
        }
        if let Some(remove_categories) = data.categories_removed {
            for c in remove_categories {
                self.categories.remove(&c);
            }
        }
        if let Some(add_categories) = data.categories {
            self.categories.extend(add_categories);
        }
        if let Some(remove_tags) = data.tags_removed {
            for t in remove_tags {
                self.tags.remove(&t);
            }
        }
        if let Some(add_tags) = data.tags {
            self.tags.extend(add_tags);
        }
        if let Some(remove_torrents) = data.torrents_removed {
            for t in remove_torrents {
                self.torrents.remove(&t);
            }
        }
        if let Some(add_torrents) = data.torrents {
            for (hash, torrent_patch) in add_torrents {
                if let Some(torrent_full) = self.torrents.get_mut(&hash) {
                    *torrent_full = merge(torrent_full, &torrent_patch).unwrap_or_else(|_| {
                        unreachable!("failed to merge torrents, but they are same type")
                    });
                } else {
                    self.torrents.insert(hash, torrent_patch);
                }
            }
        }
        if let Some(remove_trackers) = data.trackers_removed {
            for t in remove_trackers {
                self.trackers.remove(&t);
            }
        }
        if let Some(add_trackers) = data.trackers {
            self.trackers.extend(add_trackers);
        }
        if let Some(server_state) = data.server_state {
            self.server_state = merge(&self.server_state, &server_state).unwrap_or_else(|_| {
                unreachable!("failed to merge server state, but they are same type")
            });
        }
    }
}

pub struct QBittorrentDownloader {
    pub subscriber_id: i32,
    pub downloader_id: i32,
    pub endpoint_url: Url,
    pub client: Arc<Qbit>,
    pub save_path: PathBuf,
    pub wait_sync_timeout: Duration,
    pub sync_watch: watch::Sender<DateTime<Utc>>,
    pub sync_data: Arc<RwLock<QBittorrentSyncData>>,
}

impl QBittorrentDownloader {
    #[instrument(level = "debug")]
    pub async fn from_creation(
        creation: QBittorrentDownloaderCreation,
    ) -> Result<Arc<Self>, DownloaderError> {
        let endpoint_url = Url::parse(&creation.endpoint)?;

        let credential = Credential::new(creation.username, creation.password);

        let client = Qbit::new(endpoint_url.clone(), credential);

        client.login(false).await?;

        client.sync(None).await?;

        let downloader = Arc::new(Self {
            client: Arc::new(client),
            endpoint_url,
            subscriber_id: creation.subscriber_id,
            save_path: creation.save_path.into(),
            wait_sync_timeout: creation
                .wait_sync_timeout
                .unwrap_or(Duration::from_secs(10)),
            downloader_id: creation.downloader_id,
            sync_watch: watch::channel(Utc::now()).0,
            sync_data: Arc::new(RwLock::new(QBittorrentSyncData::default())),
        });

        let event_loop_me = Arc::downgrade(&downloader);

        tokio::spawn(async move { Self::start_event_loop(event_loop_me).await });

        Ok(downloader)
    }

    async fn start_event_loop(me: Weak<Self>) {
        let mut tick = 0;

        loop {
            sleep(Duration::from_millis(100)).await;
            if let Some(me) = me.upgrade() {
                if tick >= 100 {
                    let _ = me.sync_data().await.inspect_err(|e| {
                        tracing::error!(name = "sync_data", error = ?e);
                    });
                    tick = 0;
                    continue;
                }
                let count = me.sync_watch.receiver_count();
                if count > 0 && tick >= 10 {
                    let _ = me.sync_data().await.inspect_err(|e| {
                        tracing::error!(name = "sync_data", error = ?e);
                    });
                    tick = i32::max(0, tick - 10);
                } else {
                    tick += 1;
                }
            }
        }
    }

    #[instrument(level = "debug")]
    pub async fn api_version(&self) -> Result<String, DownloaderError> {
        let result = self.client.get_webapi_version().await?;
        Ok(result)
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn add_category(&self, category: &str) -> Result<(), DownloaderError> {
        self.client
            .add_category(
                NonEmptyStr::new(category)
                    .whatever_context::<_, DownloaderError>("category can not be empty")?,
                self.save_path.as_str(),
            )
            .await?;
        self.wait_sync_until(
            |sync_data| sync_data.categories.contains_key(category),
            None,
        )
        .await?;

        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn check_connection(&self) -> Result<(), DownloaderError> {
        self.api_version().await?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn set_torrents_category(
        &self,
        hashes: Vec<String>,
        category: &str,
    ) -> Result<(), DownloaderError> {
        {
            let category_no_exists = {
                let sync_data = self.sync_data.read().await;
                !sync_data.categories.contains_key(category)
            };

            if category_no_exists {
                self.add_category(category).await?;
            }
        }
        self.client
            .set_torrent_category(hashes.clone(), category)
            .await?;
        self.wait_sync_until(
            |sync_data| {
                let torrents = &sync_data.torrents;
                hashes.iter().all(|h| {
                    torrents
                        .get(h)
                        .is_some_and(|t| t.category.as_deref().is_some_and(|c| c == category))
                })
            },
            None,
        )
        .await?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn add_torrent_tags(
        &self,
        hashes: Vec<String>,
        tags: Vec<String>,
    ) -> Result<(), DownloaderError> {
        if tags.is_empty() {
            whatever!("add bittorrent tags can not be empty");
        }
        self.client
            .add_torrent_tags(hashes.clone(), tags.clone())
            .await?;
        let tag_sets = tags.iter().map(|s| s.as_str()).collect::<HashSet<&str>>();
        self.wait_sync_until(
            |sync_data| {
                let torrents = &sync_data.torrents;

                hashes.iter().all(|h| {
                    torrents.get(h).is_some_and(|t| {
                        t.tags.as_ref().is_some_and(|t| {
                            t.split(',')
                                .map(|s| s.trim())
                                .filter(|s| !s.is_empty())
                                .collect::<HashSet<&str>>()
                                .is_superset(&tag_sets)
                        })
                    })
                })
            },
            None,
        )
        .await?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn move_torrents(
        &self,
        hashes: Vec<String>,
        new_path: &str,
    ) -> Result<(), DownloaderError> {
        self.client
            .set_torrent_location(hashes.clone(), new_path)
            .await?;

        self.wait_sync_until(
            |sync_data| -> bool {
                let torrents = &sync_data.torrents;

                hashes.iter().all(|h| {
                    torrents.get(h).is_some_and(|t| {
                        t.save_path.as_deref().is_some_and(|p| {
                            path_equals_as_file_url(p, new_path)
                            .inspect_err(|error| {
                                tracing::warn!(name = "path_equals_as_file_url", error = ?error);
                            })
                            .unwrap_or(false)
                        })
                    })
                })
            },
            None,
        )
        .await?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn get_torrent_path(
        &self,
        hashes: String,
    ) -> Result<Option<String>, DownloaderError> {
        let mut torrent_list = self
            .client
            .get_torrent_list(GetTorrentListArg {
                hashes: Some(hashes),
                ..Default::default()
            })
            .await?;
        let torrent = torrent_list
            .first_mut()
            .whatever_context::<_, DownloaderError>("No bittorrent found")?;
        Ok(torrent.save_path.take())
    }

    #[instrument(level = "debug", skip(self))]
    async fn sync_data(&self) -> Result<(), DownloaderError> {
        let rid = { self.sync_data.read().await.rid };
        let sync_data_patch = self.client.sync(Some(rid)).await?;
        {
            let mut sync_data = self.sync_data.write().await;
            sync_data.patch(sync_data_patch);
        }
        let now = Utc::now();
        self.sync_watch.send_replace(now);
        Ok(())
    }

    async fn wait_sync_until<S>(
        &self,
        stop_wait_fn: S,
        timeout: Option<Duration>,
    ) -> Result<(), DownloaderError>
    where
        S: Fn(&QBittorrentSyncData) -> bool,
    {
        {
            let sync_data = &self.sync_data.read().await;
            if stop_wait_fn(sync_data) {
                return Ok(());
            }
        }

        let timeout = timeout.unwrap_or(self.wait_sync_timeout);
        let start_time = Utc::now();

        let mut receiver = self.sync_watch.subscribe();

        while let Ok(()) = receiver.changed().await {
            let has_timeout = {
                let sync_time = *receiver.borrow();
                let diff_time = sync_time - start_time;
                diff_time.num_milliseconds() > timeout.as_millis() as i64
            };
            if has_timeout {
                tracing::warn!(name = "wait_until timeout", timeout = ?timeout);
                return Err(DownloaderError::DownloadTimeoutError {
                    action: Cow::Borrowed("QBittorrentDownloader::wait_unit"),
                    timeout,
                });
            }
            {
                let sync_data = &self.sync_data.read().await;
                if stop_wait_fn(sync_data) {
                    break;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl DownloaderTrait for QBittorrentDownloader {
    type State = QBittorrentState;
    type Id = QBittorrentHash;
    type Task = QBittorrentTask;
    type Creation = QBittorrentCreation;
    type Selector = QBittorrentSelector;

    #[instrument(level = "debug", skip(self))]
    async fn add_downloads(
        &self,
        creation: <Self as DownloaderTrait>::Creation,
    ) -> Result<HashSet<<Self as DownloaderTrait>::Id>, DownloaderError> {
        let tags = {
            let mut tags = vec![TORRENT_TAG_NAME.to_string()];
            tags.extend(creation.tags);
            Some(tags.into_iter().filter(|s| !s.is_empty()).join(","))
        };

        let save_path = Some(creation.save_path.into_string());

        let sources = creation.sources;
        let hashes = HashSet::from_iter(sources.iter().map(|s| s.hash_info().to_string()));
        let (urls_source, files_source) = {
            let mut urls = vec![];
            let mut files = vec![];
            for s in sources {
                match s {
                    HashTorrentSource::MagnetUrl(MagnetUrlSource { url, .. }) => {
                        urls.push(Url::parse(&url)?)
                    }
                    HashTorrentSource::TorrentFile(TorrentFileSource {
                        payload, filename, ..
                    }) => files.push(TorrentFile {
                        filename,
                        data: payload.into(),
                    }),
                }
            }
            (
                if urls.is_empty() {
                    None
                } else {
                    Some(TorrentSource::Urls {
                        urls: Sep::from(urls),
                    })
                },
                if files.is_empty() {
                    None
                } else {
                    Some(TorrentSource::TorrentFiles { torrents: files })
                },
            )
        };

        let category = creation.category;

        if let Some(category) = category.as_deref() {
            let has_caetgory = {
                self.sync_data
                    .read()
                    .await
                    .categories
                    .contains_key(category)
            };
            if !has_caetgory {
                self.add_category(category).await?;
            }
        }

        if let Some(source) = urls_source {
            self.client
                .add_torrent(AddTorrentArg {
                    source,
                    savepath: save_path.clone(),
                    auto_torrent_management: Some(false),
                    category: category.clone(),
                    tags: tags.clone(),
                    ..Default::default()
                })
                .await?;
        }

        if let Some(source) = files_source {
            self.client
                .add_torrent(AddTorrentArg {
                    source,
                    savepath: save_path,
                    auto_torrent_management: Some(false),
                    category,
                    tags,
                    ..Default::default()
                })
                .await?;
        }
        self.wait_sync_until(
            |sync_data| {
                let torrents = &sync_data.torrents;
                hashes.iter().all(|hash| torrents.contains_key(hash))
            },
            None,
        )
        .await?;
        Ok(hashes)
    }

    async fn pause_downloads(
        &self,
        selector: <Self as DownloaderTrait>::Selector,
    ) -> Result<impl IntoIterator<Item = Self::Id>, DownloaderError> {
        <Self as TorrentDownloaderTrait>::pause_downloads(self, selector).await
    }

    async fn resume_downloads(
        &self,
        selector: <Self as DownloaderTrait>::Selector,
    ) -> Result<impl IntoIterator<Item = Self::Id>, DownloaderError> {
        <Self as TorrentDownloaderTrait>::resume_downloads(self, selector).await
    }

    async fn remove_downloads(
        &self,
        selector: <Self as DownloaderTrait>::Selector,
    ) -> Result<impl IntoIterator<Item = Self::Id>, DownloaderError> {
        <Self as TorrentDownloaderTrait>::remove_downloads(self, selector).await
    }

    #[instrument(level = "debug", skip(self))]
    async fn query_downloads(
        &self,
        selector: QBittorrentSelector,
    ) -> Result<Vec<<Self as DownloaderTrait>::Task>, DownloaderError> {
        let selector = match selector {
            QBittorrentSelector::Hash(h) => h.into(),
            QBittorrentSelector::Complex(c) => c,
        };

        let torrent_list = self.client.get_torrent_list(selector.query).await?;

        let torrent_contents = futures::future::try_join_all(torrent_list.iter().map(|s| async {
            if let Some(hash) = &s.hash {
                self.client.get_torrent_contents(hash as &str, None).await
            } else {
                Ok(vec![])
            }
        }))
        .await?;

        let tasks = torrent_list
            .into_iter()
            .zip(torrent_contents)
            .map(|(t, c)| Self::Task::from_query(t, c))
            .collect::<Result<Vec<Self::Task>, _>>()?;
        Ok(tasks)
    }
}

#[async_trait]
impl TorrentDownloaderTrait for QBittorrentDownloader {
    type IdSelector = QBittorrentHashSelector;

    #[instrument(level = "debug", skip(self))]
    async fn pause_torrents(
        &self,
        hashes: <Self as TorrentDownloaderTrait>::IdSelector,
    ) -> Result<Self::IdSelector, DownloaderError> {
        self.client.stop_torrents(hashes.clone()).await?;
        Ok(hashes)
    }

    #[instrument(level = "debug", skip(self))]
    async fn resume_torrents(
        &self,
        hashes: <Self as TorrentDownloaderTrait>::IdSelector,
    ) -> Result<<Self as TorrentDownloaderTrait>::IdSelector, DownloaderError> {
        self.client.start_torrents(hashes.clone()).await?;
        Ok(hashes)
    }

    #[instrument(level = "debug", skip(self))]
    async fn remove_torrents(
        &self,
        hashes: <Self as TorrentDownloaderTrait>::IdSelector,
    ) -> Result<Self::IdSelector, DownloaderError> {
        self.client
            .delete_torrents(hashes.clone(), Some(true))
            .await?;
        self.wait_sync_until(
            |sync_data| -> bool {
                let torrents = &sync_data.torrents;
                hashes.iter().all(|h| !torrents.contains_key(h))
            },
            None,
        )
        .await?;
        Ok(hashes)
    }
}

impl Debug for QBittorrentDownloader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QBittorrentDownloader")
            .field("subscriber_id", &self.subscriber_id)
            .field("client", &self.endpoint_url.as_str())
            .finish()
    }
}
