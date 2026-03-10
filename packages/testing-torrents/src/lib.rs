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

pub async fn create_testcontainers() -> Result<
    testcontainers::ContainerRequest<testcontainers::GenericImage>,
    testcontainers::TestcontainersError,
> {
    let container = GenericImage::new("ghcr.io/apeiraco/konobangu-testing-torrents", "latest")
        .with_wait_for(WaitFor::message_on_stdout("Listening on"))
        .with_mapped_port(6080, ContainerPort::Tcp(6080))
        .with_mapped_port(6081, ContainerPort::Tcp(6081))
        .with_mapped_port(6082, ContainerPort::Tcp(6082))
        .with_default_log_consumer()
        .with_prune_existed_label("konobangu", "testing-torrents", true, true)
        .await?;

    Ok(container)
}
