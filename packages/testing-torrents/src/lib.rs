use serde::{Deserialize, Serialize};
use testcontainers::{
    GenericImage,
    core::{ContainerPort, WaitFor},
};
use testcontainers_ext::{ImageDefaultLogConsumerExt, ImagePruneExistedLabelExt};
use testcontainers_modules::testcontainers::ImageExt;

#[derive(Serialize)]
pub struct TestingTorrentFileItem {
    pub path: String,
    pub size: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestTorrentRequest {
    pub id: String,
    pub file_list: Vec<TestingTorrentFileItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestTorrentResponse {
    pub torrent_url: String,
    pub magnet_url: String,
    pub hash: String,
}

#[derive(Debug)]
pub struct TestingTorrentsInstance {
    pub req: testcontainers::ContainerRequest<testcontainers::GenericImage>,
    pub api_port: u16,
    pub tracker_port: u16,
    pub seeding_port: u16,
}

fn get_free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

pub async fn create_testcontainers() -> Result<
    TestingTorrentsInstance,
    testcontainers::TestcontainersError,
> {
    let api_port = get_free_port();
    let tracker_port = get_free_port();
    let seeding_port = get_free_port();

    let container = GenericImage::new("ghcr.io/apeiraco/konobangu-testing-torrents", "latest")
        .with_wait_for(WaitFor::message_on_stdout("Listening on"))
        .with_env_var("API_PORT", api_port.to_string())
        .with_env_var("TRACKER_PORT", tracker_port.to_string())
        .with_env_var("SEEDING_PORT", seeding_port.to_string())
        .with_mapped_port(api_port, ContainerPort::Tcp(api_port))
        .with_mapped_port(tracker_port, ContainerPort::Tcp(tracker_port))
        .with_mapped_port(seeding_port, ContainerPort::Tcp(seeding_port))
        .with_default_log_consumer()
        .with_prune_existed_label("konobangu", "testing-torrents", false, false)
        .await?;

    Ok(TestingTorrentsInstance {
        req: container,
        api_port,
        tracker_port,
        seeding_port,
    })
}
