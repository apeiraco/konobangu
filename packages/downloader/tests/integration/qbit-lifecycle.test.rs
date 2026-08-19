use std::time::Duration;

use downloader::{
  bittorrent::{downloader::TorrentDownloaderTrait, source::HashTorrentSource},
  core::{DownloadIdSelectorTrait, DownloaderTrait},
  qbit::{
    QBittorrentDownloader, QBittorrentDownloaderCreation,
    task::{QBittorrentCreation, QBittorrentHashSelector, QBittorrentSelector},
  },
};

#[derive(Debug)]
pub struct QbitTestcontainersInstance {
  pub req: testcontainers::ContainerRequest<testcontainers::GenericImage>,
  pub webui_port: u16,
}

pub async fn create_qbit_testcontainers() -> anyhow::Result<QbitTestcontainersInstance> {
  use testcontainers::{
    GenericImage,
    core::{ContainerPort, WaitFor},
  };
  use testcontainers_ext::ImageDefaultLogConsumerExt;
  use testcontainers_modules::testcontainers::ImageExt;

  let webui_port = 8080;
  let torrenting_port = 6881;

  let container = GenericImage::new(
    "linuxserver/qbittorrent",
    "5.2.4_v2.0.15-ls479@sha256:b522f9f4b769f8f36d49d22d5eb6a92e9aa18904c6a1830b1439df511ec21983",
  )
  .with_wait_for(WaitFor::message_on_stderr("Connection to localhost"))
  // Docker reserves host ports atomically; probing and releasing a local
  // socket before container startup races with other test containers.
  .with_exposed_port(ContainerPort::Tcp(webui_port))
  .with_exposed_port(ContainerPort::Tcp(torrenting_port))
  .with_env_var("WEBUI_PORT", webui_port.to_string())
  .with_env_var("TZ", "Asia/Singapore")
  .with_host("host.docker.internal", testcontainers::core::Host::HostGateway)
  .with_env_var("TORRENTING_PORT", torrenting_port.to_string())
  // This fixture deliberately maps a random host port to the fixed service
  // port. Disable only qBittorrent's port-equality check; authentication and
  // CSRF protection remain enabled and the wrong-password case is asserted.
  .with_copy_to(
    "/config/qBittorrent/qBittorrent.conf",
    b"[Preferences]\nWebUI\\HostHeaderValidation=false\n".to_vec(),
  )
  .with_default_log_consumer();

  Ok(QbitTestcontainersInstance { req: container, webui_port })
}

#[tokio::test(flavor = "multi_thread")]
async fn downloader_qbit_local_lifecycle_sync_auth_timeout_shutdown() -> anyhow::Result<()> {
  use anyhow::Context;
  use downloader::core::{DownloadSimpleState, DownloadStateTrait, DownloadTaskTrait};
  use testcontainers::runners::AsyncRunner;
  use tokio::io::AsyncReadExt;
  let fixture = testing_torrents::LocalTestingTorrents::start().await?;
  let torrent = fixture.torrent().await?;
  let image = create_qbit_testcontainers().await?;
  let container = image.req.start().await?;
  let webui_port = container.get_host_port_ipv4(image.webui_port).await?;
  for command in [vec!["mkdir", "-p", "/downloads/fixture"], vec!["chmod", "0777", "/downloads/fixture"]] {
    let status = tokio::process::Command::new("docker")
      .args(["exec", container.id()])
      .args(command)
      .status()
      .await?;
    anyhow::ensure!(
      status.success(),
      "Owned download directory must be writable by the fixture's unprivileged qBittorrent process"
    );
  }
  let mut logs = String::new();
  container.stdout(false).read_to_string(&mut logs).await?;
  let username = logs
    .lines()
    .find(|line| line.contains("The WebUI administrator username is"))
    .and_then(|line| line.split_whitespace().last())
    .ok_or_else(|| anyhow::anyhow!("Fixture username missing"))?;
  let password = logs
    .lines()
    .find(|line| line.contains("A temporary password is provided for"))
    .and_then(|line| line.split_whitespace().last())
    .ok_or_else(|| anyhow::anyhow!("Fixture password missing"))?;
  let creation = |password: &str| QBittorrentDownloaderCreation {
    endpoint: format!("http://127.0.0.1:{webui_port}"),
    username: username.into(),
    password: password.into(),
    save_path: "/downloads/fixture".into(),
    subscriber_id: 1,
    downloader_id: 1,
    wait_sync_timeout: Some(Duration::from_secs(5)),
  };
  assert!(QBittorrentDownloader::from_creation(creation("wrong-fixture-password")).await.is_err());
  let downloader = QBittorrentDownloader::from_creation(creation(password)).await?;
  let result = async {
    let version = downloader.client.get_version().await?;
    eprintln!("qBittorrent lifecycle fixture version: {}", version.trim());
    assert!(version.trim_start_matches('v').starts_with('5'));
    let original = reqwest::get(&torrent.torrent_url).await?.error_for_status()?.bytes().await?;
    // Only top-level tracker/webseed URL strings change; the info dictionary
    // and known hash stay intact. Docker reaches the local fixture through
    // its host.
    let mut payload = original.to_vec();
    let torrent_url = url::Url::parse(&torrent.torrent_url)?;
    let name = torrent_url
      .path_segments()
      .and_then(Iterator::last)
      .ok_or_else(|| anyhow::anyhow!("Fixture torrent name missing"))?
      .trim_end_matches(".torrent");
    for port in [fixture.api_port, fixture.tracker_port] {
      let from = format!("http://127.0.0.1:{port}");
      let to = format!("http://host.docker.internal:{port}");
      // URL suffixes are part of each bencoded string length.
      for suffix in ["/announce", &format!("/api/static/{name}/")] {
        let old = format!("{}:{}{}", from.len() + suffix.len(), from, suffix).into_bytes();
        let new = format!("{}:{}{}", to.len() + suffix.len(), to, suffix).into_bytes();
        while let Some(at) = payload.windows(old.len()).position(|w| w == old) {
          payload.splice(at..at + old.len(), new.clone());
        }
      }
    }
    let source = HashTorrentSource::TorrentFile(downloader::bittorrent::source::TorrentFileSource::from_bytes(
      "fixture.torrent".into(),
      payload.into(),
      None,
    )?);
    let add = || QBittorrentCreation {
      save_path: "/downloads/fixture".into(),
      tags: vec![],
      category: None,
      sources: vec![source.clone()],
    };
    downloader.add_downloads(add()).await.context("Adding the first fixture torrent")?;
    downloader.add_downloads(add()).await.context("Adding an existing fixture torrent again")?;
    let selector = || QBittorrentSelector::Hash(QBittorrentHashSelector::from_id(torrent.hash.clone()));
    assert_eq!(downloader.query_downloads(selector()).await?.len(), 1);
    let rid = downloader.sync_data.read().await.rid;
    assert!(rid > 0);
    downloader.sync_data().await?;
    assert!(downloader.sync_data.read().await.rid >= rid);
    downloader.pause_torrents(vec![torrent.hash.clone()].into()).await?;
    let mut pause_state = None;
    tokio::time::timeout(Duration::from_secs(5), async {
      loop {
        let tasks = downloader.query_downloads(selector()).await?;
        pause_state = tasks[0].torrent.state.clone();
        if tasks[0].state().to_download_state() == DownloadSimpleState::Paused {
          break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
      }
      Ok::<_, anyhow::Error>(())
    })
    .await
    .with_context(|| format!("Paused state timed out, last state: {pause_state:?}"))??;
    downloader.resume_torrents(vec![torrent.hash.clone()].into()).await?;
    tokio::time::timeout(Duration::from_secs(5), async {
      loop {
        if downloader.query_downloads(selector()).await?[0].state().to_download_state() != DownloadSimpleState::Paused {
          break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
      }
      Ok::<_, anyhow::Error>(())
    })
    .await
    .context("Resumed state timed out")??;
    tokio::time::timeout(Duration::from_secs(30), async {
      loop {
        let tasks = downloader.query_downloads(selector()).await?;
        if tasks[0].progress().is_some_and(|progress| progress >= 1.0) {
          break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
      }
      Ok::<_, anyhow::Error>(())
    })
    .await
    .context("Complete download timed out")??;
    let content = downloader.query_downloads(selector()).await?[0].contents[0].name.clone();
    let file = format!("/downloads/fixture/{content}");
    let exists = || async {
      let output = tokio::process::Command::new("docker")
        .args(["exec", container.id(), "test", "-f", &file])
        .output()
        .await?;
      Ok::<_, anyhow::Error>(output.status.success())
    };
    assert!(exists().await?);
    downloader.remove_torrents_with_files(vec![torrent.hash.clone()].into(), false).await?;
    assert!(downloader.query_downloads(selector()).await?.is_empty());
    assert!(exists().await?, "Keep-files deletion must preserve downloaded content");
    assert!(!downloader.sync_data.read().await.torrents.contains_key(&torrent.hash));
    downloader.add_downloads(add()).await.context("Adding the fixture after keep-files deletion")?;
    downloader.remove_torrents(vec![torrent.hash.clone()].into()).await?;
    downloader.remove_torrents(vec![torrent.hash.clone()].into()).await?;
    tokio::time::timeout(Duration::from_secs(5), async {
      while exists().await? {
        tokio::time::sleep(Duration::from_millis(50)).await;
      }
      Ok::<_, anyhow::Error>(())
    })
    .await
    .context("Removed downloaded file timed out")??;
    // A silent endpoint proves the client has a bounded request, independently
    // of background sync notifications.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let silent = tokio::spawn(async move {
      let (socket, _) = listener.accept().await.unwrap();
      let _socket = socket;
      std::future::pending::<()>().await;
    });
    let mut options = creation(password);
    options.endpoint = format!("http://{address}");
    options.wait_sync_timeout = Some(Duration::from_millis(100));
    let timed = tokio::time::timeout(Duration::from_secs(2), QBittorrentDownloader::from_creation(options)).await;
    silent.abort();
    assert!(timed?.is_err());
    Ok::<_, anyhow::Error>(())
  }
  .await;
  tokio::time::timeout(Duration::from_secs(6), downloader.shutdown()).await?;
  result
}
