#[test]
fn downloader_qbit_incremental_sync_rid_removals_and_full_update() {
  let mut data = super::downloader::QBittorrentSyncData::default();
  let patch = |json| serde_json::from_value::<qbit_rs::model::SyncData>(json).unwrap();
  data.patch(patch(
    serde_json::json!({"rid":1,"full_update":true,"torrents":{"a":{"name":"first","progress":0.5}},"tags":["old"],"server_state":{"old":1}}),
  ));
  data.patch(patch(
    serde_json::json!({"rid":2,"torrents":{"a":{"progress":1.0},"b":{"name":"second"}},"tags_removed":["old"],"tags":["new"]}),
  ));
  assert_eq!(data.rid, 2);
  assert_eq!(data.torrents["a"].name.as_deref(), Some("first"));
  assert_eq!(data.torrents["a"].progress, Some(1.0));
  assert!(!data.tags.contains("old"));
  data.patch(patch(serde_json::json!({"rid":3,"torrents_removed":["a"]})));
  assert!(!data.torrents.contains_key("a"));
  data.patch(patch(serde_json::json!({"rid":4,"full_update":true,"torrents":{},"server_state":{"new":2}})));
  assert!(data.torrents.is_empty());
  assert!(data.tags.is_empty());
  assert!(!data.server_state.contains_key("old"));
  assert_eq!(data.rid, 4);
}

async fn mocked_add_response(
  status: usize,
  requested: usize,
  present: usize,
  magnet: bool,
) -> anyhow::Result<Result<std::collections::HashSet<String>, crate::DownloaderError>> {
  use std::{collections::HashSet, time::Duration};

  use serde_json::json;

  use crate::{
    bittorrent::source::{HashTorrentSource, HashTorrentSourceTrait, TorrentFileSource},
    core::DownloaderTrait,
    qbit::{QBittorrentDownloader, QBittorrentDownloaderCreation, task::QBittorrentCreation},
  };

  let sources = (0..requested)
    .map(|index| {
      let name = format!("fixture-{index}.bin");
      let mut payload = format!("d4:infod6:lengthi1e4:name{}:{}12:piece lengthi16384e6:pieces20:", name.len(), name).into_bytes();
      payload.extend([0; 20]);
      payload.extend(b"ee");
      let source = HashTorrentSource::TorrentFile(TorrentFileSource::from_bytes("fixture.torrent".into(), payload.into(), None)?);
      if magnet {
        HashTorrentSource::from_magnet_url(format!("magnet:?xt=urn:btih:{}", source.hash_info()))
      } else {
        Ok(source)
      }
    })
    .collect::<Result<Vec<_>, _>>()?;
  let hashes: Vec<_> = sources.iter().map(|source| source.hash_info().into_owned()).collect();
  let mut server = mockito::Server::new_async().await;
  let login = server
    .mock("POST", "/api/v2/auth/login")
    .with_status(200)
    .with_header("set-cookie", "SID=fixture; Path=/")
    .with_body("Ok.")
    .create_async()
    .await;
  // All hashes are deliberately cached, even when absent on the server.
  let sync = server
    .mock("GET", "/api/v2/sync/maindata")
    .match_query(mockito::Matcher::Any)
    .with_status(200)
    .with_body(
      json!({"rid":1,"full_update":true,"torrents": hashes.iter().map(|hash| (hash.clone(), json!({"hash":hash}))).collect::<serde_json::Map<_, _>>()})
        .to_string(),
    )
    .expect_at_least(1)
    .create_async()
    .await;
  let add = server.mock("POST", "/api/v2/torrents/add").with_status(status).create_async().await;
  let info = server
    .mock("GET", "/api/v2/torrents/info")
    .match_query(mockito::Matcher::Any)
    .with_status(200)
    .with_body(json!(hashes.iter().take(present).map(|hash| json!({"hash":hash})).collect::<Vec<_>>()).to_string())
    .expect(usize::from(status == 409))
    .create_async()
    .await;
  let downloader = QBittorrentDownloader::from_creation(QBittorrentDownloaderCreation {
    endpoint: server.url(),
    username: "fixture".into(),
    password: "fixture".into(),
    save_path: "/downloads/fixture".into(),
    subscriber_id: 1,
    downloader_id: 1,
    wait_sync_timeout: Some(Duration::from_secs(2)),
  })
  .await?;
  let result = downloader
    .add_downloads(QBittorrentCreation {
      save_path: "/downloads/fixture".into(),
      tags: vec![],
      category: None,
      sources,
    })
    .await;
  downloader.shutdown().await;
  login.assert_async().await;
  sync.assert_async().await;
  add.assert_async().await;
  info.assert_async().await;
  if let Ok(ids) = &result {
    assert_eq!(ids, &HashSet::from_iter(hashes));
  }
  Ok(result)
}

#[tokio::test]
async fn qbit_add_conflict_requires_all_hashes_on_server() -> anyhow::Result<()> {
  assert!(mocked_add_response(409, 1, 1, false).await?.is_ok());
  for (requested, present) in [(1, 0), (2, 1)] {
    assert!(matches!(
      mocked_add_response(409, requested, present, false).await?,
      Err(crate::DownloaderError::QBitAPIError {
        source: qbit_rs::Error::ApiError(qbit_rs::ApiError::TorrentAddFailed)
      })
    ));
  }
  Ok(())
}

#[tokio::test]
async fn qbit_add_magnet_conflicts_and_other_errors() -> anyhow::Result<()> {
  assert!(mocked_add_response(409, 1, 1, true).await?.is_ok());
  assert!(matches!(
    mocked_add_response(409, 1, 0, true).await?,
    Err(crate::DownloaderError::QBitAPIError {
      source: qbit_rs::Error::UnknownHttpCode(code)
    }) if code.as_u16() == 409
  ));
  assert!(matches!(
    mocked_add_response(415, 1, 1, false).await?,
    Err(crate::DownloaderError::QBitAPIError {
      source: qbit_rs::Error::ApiError(qbit_rs::ApiError::TorrentFileInvalid)
    })
  ));
  Ok(())
}
