pub mod block;
pub mod error;
pub mod graph_uri;
pub mod ids;
pub mod mapping;
pub mod neo4j_utils;
pub mod pb;

pub use ids::indexer_ids;
pub use ids::network_ids;
pub use ids::system_ids;

pub use mapping::entity;
pub use mapping::relation;

pub use neo4rs;

pub use grc20_macros::{entity, relation};

#[cfg(test)]
pub(crate) mod test_utils {
    use testcontainers::{
        core::{IntoContainerPort, WaitFor},
        runners::AsyncRunner,
        GenericImage, ImageExt,
    };

    const BOLT_PORT: u16 = 7687;
    const HTTP_PORT: u16 = 7474;

    pub async fn setup_neo4j() -> (testcontainers::ContainerAsync<GenericImage>, neo4rs::Graph) {
        // Setup a local Neo4J container for testing
        let container = GenericImage::new("neo4j", "2025.01.0-community")
            .with_wait_for(WaitFor::Duration {
                length: std::time::Duration::from_secs(5),
            })
            .with_exposed_port(BOLT_PORT.tcp())
            .with_exposed_port(HTTP_PORT.tcp())
            .with_env_var("NEO4J_AUTH", "none")
            .start()
            .await
            .expect("Failed to start Neo4J container");

        let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
        let host = container.get_host().await.unwrap().to_string();

        let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "", "")
            .await
            .unwrap();

        (container, neo4j)
    }
}
