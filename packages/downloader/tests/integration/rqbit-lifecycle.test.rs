#[tokio::test(flavor = "multi_thread")]
async fn downloader_rqbit_local_file_lifecycle_repeated_add_delete_modes_shutdown() -> anyhow::Result<()> {
  use std::{str::FromStr, sync::Arc, time::Duration};

  use downloader::{
    bittorrent::source::{HashTorrentSource, TorrentFileSource},
    core::{DownloadSimpleState, DownloadStateTrait, DownloadTaskTrait},
    rqbit::downloader::{RqbitDownloader, RqbitDownloaderCreation},
  };
  let fixture = testing_torrents::LocalTestingTorrents::start().await?;
  let torrent = fixture.torrent().await?;
  let output = fixture.directory.join("rqbit-output");
  std::fs::create_dir_all(&output)?;
  let options = librqbit::SessionOptions {
    dht: None,
    disable_local_service_discovery: true,
    ..Default::default()
  };
  let downloader = RqbitDownloader::from_creation_with_options(
    RqbitDownloaderCreation {
      save_path: output.to_string_lossy().into(),
      subscriber_id: 1,
      downloader_id: 1,
    },
    options,
  )
  .await?;
  let payload = reqwest::get(&torrent.torrent_url).await?.error_for_status()?.bytes().await?;
  let source = HashTorrentSource::TorrentFile(TorrentFileSource::from_bytes("fixture.torrent".into(), payload, None)?);
  let hash = librqbit_core::Id20::from_str(&torrent.hash)?;
  let add = || {
    Some(librqbit::AddTorrentOptions {
      overwrite: true,
      output_folder: Some(output.to_string_lossy().into()),
      initial_peers: Some(vec![format!("127.0.0.1:{}", fixture.seeding_port).parse().unwrap()]),
      ..Default::default()
    })
  };
  let result = async {
    downloader.add_torrent(source.clone(), add()).await?;
    downloader.add_torrent(source.clone(), add()).await?;
    tokio::time::timeout(Duration::from_secs(15), async {
      loop {
        if downloader.query_torrent(hash)?.state().to_download_state() == DownloadSimpleState::Completed {
          break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
      }
      Ok::<_, anyhow::Error>(())
    })
    .await??;
    fn known_file(root: &std::path::Path) -> Option<std::path::PathBuf> {
      for entry in std::fs::read_dir(root).ok()? {
        let path = entry.ok()?.path();
        if path.is_dir() {
          if let Some(found) = known_file(&path) {
            return Some(found);
          }
        } else if path.file_name()?.to_str() == Some("known.bin") {
          return Some(path);
        }
      }
      None
    }
    let path = known_file(&output).ok_or_else(|| anyhow::anyhow!("Actual downloaded file missing"))?;
    assert_eq!(std::fs::read(&path)?, vec![0; 4096]);
    downloader.pause_torrent(hash).await?;
    assert_eq!(downloader.query_torrent(hash)?.state().to_download_state(), DownloadSimpleState::Paused);
    downloader.resume_torrent(hash).await?;
    downloader.delete_torrent_with_files(hash, false).await?;
    assert!(path.exists());
    assert!(downloader.query_torrent(hash).is_err());
    downloader.add_torrent(source, add()).await?;
    downloader.delete_torrent_with_files(hash, true).await?;
    assert!(!path.exists());
    downloader.delete_torrent_with_files(hash, true).await?;
    assert!(TorrentFileSource::from_bytes("invalid.torrent".into(), bytes::Bytes::from_static(b"invalid"), None).is_err());
    Ok::<_, anyhow::Error>(())
  }
  .await;
  tokio::time::timeout(Duration::from_secs(5), downloader.shutdown()).await?;
  let weak = Arc::downgrade(&downloader);
  drop(downloader);
  assert!(weak.upgrade().is_none());
  result
}
