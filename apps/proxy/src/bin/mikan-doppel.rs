use std::time::Duration;

use recorder::{errors::RecorderResult, test_utils::mikan::MikanMockServer};
use tracing::Level;

#[tokio::main]
async fn main() -> RecorderResult<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let mut mikan_server = MikanMockServer::new_with_port(5005).await.unwrap();

    let _resources_mock = mikan_server.mock_resources_with_doppel();

    let _login_mock = mikan_server.mock_get_login_page();

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
