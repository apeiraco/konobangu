use bollard::query_parameters::{ListContainersOptions, PruneContainersOptions, StopContainerOptions};
use bollard::Docker;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let client = Docker::connect_with_local_defaults()
        .expect("Failed to connect to docker daemon");
        
    let scope = env!("CARGO_PKG_NAME");
    tracing::info!("Cleaning testcontainers with scope: {}", scope);
    
    let mut filters = HashMap::<String, Vec<String>>::new();
    filters.insert(
        String::from("label"),
        vec![format!("{}.testcontainers.scope={}", scope, scope)],
    );

    let result = client
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters: Some(filters.clone()),
            ..Default::default()
        }))
        .await
        .expect("Failed to list containers");

    let remove_containers = result
        .into_iter()
        .flat_map(|c| c.id)
        .collect::<Vec<_>>();

    if remove_containers.is_empty() {
        tracing::info!("No testcontainers found to clean.");
        return;
    }

    tracing::info!("Stopping {} testcontainer(s)...", remove_containers.len());

    futures::future::join_all(
        remove_containers
            .iter()
            .map(|c| client.stop_container(c, None::<StopContainerOptions>)),
    )
    .await;

    // Prune stopped containers
    client
        .prune_containers(Some(PruneContainersOptions {
            filters: Some(filters),
        }))
        .await
        .expect("Failed to prune containers");

    tracing::info!("Successfully pruned testcontainers.");
}
