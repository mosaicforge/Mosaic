use grc20_core::{
    mapping::{triple, Query},
    neo4rs,
};
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

    // Create indexes
    neo4j
        .run(neo4rs::query(
            "CREATE INDEX entity_id_index FOR (e:Entity) ON (e.id)",
        ))
        .await
        .unwrap();
    neo4j
        .run(neo4rs::query(
            "CREATE INDEX relation_id_index FOR (r:Relation) ON (r.id)",
        ))
        .await
        .unwrap();

    // Give time to neo4j to create the indices
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Bootstrap the database
    let block = crate::common::create_block_metadata();
    triple::insert_many(&neo4j, &block, "ROOT", "0")
        .triples(sink::bootstrap::boostrap_indexer::triples())
        .send()
        .await
        .expect("Failed to bootstrap indexer");

    (container, neo4j)
}
