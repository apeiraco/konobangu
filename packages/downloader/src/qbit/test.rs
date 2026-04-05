use std::time::Duration;

use chrono::Utc;
use qbit_rs::model::{GetTorrentListArg, TorrentFilter as QbitTorrentFilter};
use quirks_path::Path;
use snafu::OptionExt;

use crate::{
    DownloaderError,
    bittorrent::{
        downloader::TorrentDownloaderTrait, source::HashTorrentSource, task::TorrentTaskTrait,
    },
    core::{DownloadIdSelectorTrait, DownloaderTrait},
    qbit::{
        QBittorrentDownloader, QBittorrentDownloaderCreation,
        task::{
            QBittorrentComplexSelector, QBittorrentCreation, QBittorrentHashSelector,
            QBittorrentSelector, QBittorrentTask,
        },
    },
    utils::path_equals_as_file_url,
};

fn get_tmp_qbit_test_folder() -> &'static str {
    if cfg!(all(windows, not(feature = "testcontainers"))) {
        "C:\\Windows\\Temp\\konobangu\\qbit"
    } else {
        "/tmp/konobangu/qbit"
    }
}

#[derive(Debug)]
pub struct QbitTestcontainersInstance {
    pub req: testcontainers::ContainerRequest<testcontainers::GenericImage>,
    pub webui_port: u16,
    pub torrenting_port: u16,
}

fn get_free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[cfg(feature = "testcontainers")]
pub async fn create_qbit_testcontainers()
-> anyhow::Result<QbitTestcontainersInstance> {
    use testcontainers::{
        GenericImage,
        core::{
            ContainerPort,
            WaitFor,
        },
    };
    use testcontainers_ext::{ImageDefaultLogConsumerExt, ImagePruneExistedLabelExt};
    use testcontainers_modules::testcontainers::ImageExt;

    let webui_port = get_free_port();
    let torrenting_port = get_free_port();

    let container = GenericImage::new("linuxserver/qbittorrent", "latest")
        .with_wait_for(WaitFor::message_on_stderr("Connection to localhost"))
        .with_env_var("WEBUI_PORT", webui_port.to_string())
        .with_env_var("TZ", "Asia/Singapore")
        .with_env_var("TORRENTING_PORT", torrenting_port.to_string())
        .with_mapped_port(torrenting_port, ContainerPort::Tcp(torrenting_port))
        .with_mapped_port(webui_port, ContainerPort::Tcp(webui_port))
        .with_default_log_consumer()
        .with_prune_existed_label(env!("CARGO_PKG_NAME"), "qbit-downloader", false, false)
        .await?;

    Ok(QbitTestcontainersInstance {
        req: container,
        webui_port,
        torrenting_port,
    })
}

#[cfg(not(feature = "testcontainers"))]
#[tokio::test]
async fn test_qbittorrent_downloader() {
    let hash = "47ee2d69e7f19af783ad896541a07b012676f858".to_string();
    let torrent_url = format!("https://mikanani.me/Download/20240301/{}.torrent", hash);
    let _ = test_qbittorrent_downloader_impl(torrent_url, hash, None, None, 10721).await;
}

#[cfg(feature = "testcontainers")]
#[tokio::test(flavor = "multi_thread")]
async fn test_qbittorrent_downloader() -> anyhow::Result<()> {
    use testcontainers::runners::AsyncRunner;
    use testing_torrents::{TestTorrentRequest, TestTorrentResponse, TestingTorrentFileItem};
    use tokio::io::AsyncReadExt;

    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();

    let torrents_image = testing_torrents::create_testcontainers().await?;
    let _torrents_container = torrents_image.req.start().await?;

    let torrents_req = TestTorrentRequest {
        id: "f10ebdda-dd2e-43f8-b80c-bf0884d071c4".into(),
        file_list: vec![TestingTorrentFileItem {
            path: "[Nekomoe kissaten&LoliHouse] Boku no Kokoro no Yabai Yatsu - 20 [WebRip 1080p \
                   HEVC-10bit AAC ASSx2].mkv"
                .into(),
            size: 1024,
        }],
    };

    let torrent_res: TestTorrentResponse = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{}/api/torrents/mock", torrents_image.api_port))
        .json(&torrents_req)
        .send()
        .await?
        .json()
        .await?;

    let qbit_image = create_qbit_testcontainers().await?;
    let qbit_container = qbit_image.req.start().await?;

    let mut logs = String::new();

    qbit_container
        .stdout(false)
        .read_to_string(&mut logs)
        .await?;

    let username = logs
        .lines()
        .find_map(|line| {
            if line.contains("The WebUI administrator username is") {
                line.split_whitespace().last()
            } else {
                None
            }
        })
        .expect("should have username")
        .trim();

    let password = logs
        .lines()
        .find_map(|line| {
            if line.contains("A temporary password is provided for") {
                line.split_whitespace().last()
            } else {
                None
            }
        })
        .expect("should have password")
        .trim();

    tracing::info!(username, password);

    test_qbittorrent_downloader_impl(
        torrent_res.torrent_url,
        torrent_res.hash,
        Some(username),
        Some(password),
        qbit_image.webui_port,
    )
    .await?;

    Ok(())
}

async fn test_qbittorrent_downloader_impl(
    torrent_url: String,
    torrent_hash: String,
    username: Option<&str>,
    password: Option<&str>,
    webui_port: u16,
) -> anyhow::Result<()> {
    let http_client = fetch::test_util::build_testing_http_client()?;
    let base_save_path = Path::new(get_tmp_qbit_test_folder());

    let downloader = QBittorrentDownloader::from_creation(QBittorrentDownloaderCreation {
        endpoint: format!("http://127.0.0.1:{}", webui_port),
        password: password.unwrap_or_default().to_string(),
        username: username.unwrap_or_default().to_string(),
        subscriber_id: 0,
        save_path: base_save_path.to_string(),
        downloader_id: 0,
        wait_sync_timeout: Some(Duration::from_secs(3)),
    })
    .await?;

    downloader.check_connection().await?;

    downloader
        .remove_torrents(vec![torrent_hash.clone()].into())
        .await?;

    let torrent_source =
        HashTorrentSource::from_url_and_http_client(&http_client, torrent_url).await?;

    let folder_name = format!("torrent_test_{}", Utc::now().timestamp());
    let save_path = base_save_path.join(&folder_name);

    let torrent_creation = QBittorrentCreation {
        save_path,
        tags: vec![],
        sources: vec![torrent_source],
        category: None,
    };

    downloader.add_downloads(torrent_creation).await?;

    let get_torrent = async || -> Result<QBittorrentTask, DownloaderError> {
        let torrent_infos = downloader
            .query_downloads(QBittorrentSelector::Hash(QBittorrentHashSelector::from_id(
                torrent_hash.clone(),
            )))
            .await?;

        let result = torrent_infos
            .into_iter()
            .find(|t| t.hash_info() == torrent_hash)
            .whatever_context::<_, DownloaderError>("no bittorrent")?;

        Ok(result)
    };

    let target_torrent = get_torrent().await?;

    let files = target_torrent.contents;

    assert!(!files.is_empty());

    let first_file = files.first().expect("should have first file");
    assert!(
        &first_file.name.ends_with(r#"[Nekomoe kissaten&LoliHouse] Boku no Kokoro no Yabai Yatsu - 20 [WebRip 1080p HEVC-10bit AAC ASSx2].mkv"#)
    );

    let test_tag = "test_tag".to_string();

    downloader
        .add_torrent_tags(vec![torrent_hash.clone()], vec![test_tag.clone()])
        .await?;

    let target_torrent = get_torrent().await?;

    assert!(target_torrent.tags().any(|s| s == test_tag));

    let test_category = format!("test_category_{}", Utc::now().timestamp());

    downloader
        .set_torrents_category(vec![torrent_hash.clone()], &test_category)
        .await?;

    let target_torrent = get_torrent().await?;

    assert_eq!(
        Some(test_category.as_str()),
        target_torrent.category().as_deref()
    );

    let moved_torrent_path = base_save_path.join(format!("moved_{}", Utc::now().timestamp()));

    downloader
        .move_torrents(vec![torrent_hash.clone()], moved_torrent_path.as_str())
        .await?;

    let target_torrent = get_torrent().await?;

    let actual_content_path = &target_torrent
        .torrent
        .save_path
        .expect("failed to get actual save path");

    assert!(
        path_equals_as_file_url(actual_content_path, moved_torrent_path)
            .expect("failed to compare actual torrent path and found expected torrent path")
    );

    downloader
        .remove_torrents(vec![torrent_hash.clone()].into())
        .await?;

    let torrent_infos1 = downloader
        .query_downloads(QBittorrentSelector::Complex(QBittorrentComplexSelector {
            query: GetTorrentListArg::builder()
                .filter(QbitTorrentFilter::All)
                .build(),
        }))
        .await?;

    assert!(torrent_infos1.is_empty());

    tracing::info!("test finished");

    Ok(())
}
