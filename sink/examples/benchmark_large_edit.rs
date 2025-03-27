use grc20_core::{block::BlockMetadata, neo4rs, pb::geo};
use sink::events::EventHandler;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    GenericImage, ImageExt,
};

const BOLT_PORT: u16 = 7687;
const HTTP_PORT: u16 = 7474;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let container = GenericImage::new("neo4j", "2025.01.0-community")
        .with_wait_for(WaitFor::Duration {
            length: std::time::Duration::from_secs(10),
        })
        .with_exposed_port(BOLT_PORT.tcp())
        .with_exposed_port(HTTP_PORT.tcp())
        .with_env_var("NEO4J_AUTH", "none")
        .with_env_var("NEO4J_server_memory_pagecache_size", "2G")
        .with_env_var("NEO4J_server_memory_heap_initial__size", "2G")
        .with_env_var("NEO4J_server_memory_heap_max__size", "4G")
        .start()
        .await
        .expect("Failed to start Neo 4J container");

    println!("Neo4J container started");

    let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
    let host = container.get_host().await.unwrap().to_string();

    let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
        .await
        .unwrap();

    // Create indexes
    neo4j
        .run(neo4rs::query(
            "CREATE INDEX entity_id_index FOR (e:Entity) ON (e.id)",
        ))
        .await?;
    neo4j
        .run(neo4rs::query(
            "CREATE INDEX relation_id_index FOR (r:Relation) ON (r.id)",
        ))
        .await?;
    tokio::time::sleep(std::time::Duration::from_secs(10)).await; // Give time to neo4j to create the indices

    println!("Neo4J database reset");

    let sink = EventHandler::new(neo4j);

    let block = BlockMetadata::default();

    let space_created_event = geo::GeoSpaceCreated {
        space_address: "0x4838b106fce9647bdf1e7877bf73ce8b0bad5f97".to_string(),
        dao_address: "0x4838b106fce9647bdf1e7877bf73ce8b0bad5f97".to_string(),
    };

    let edit_event = geo::EditPublished {
        content_uri: "ipfs://bafybeibvvght7ujamrdxsiobfkc32g7ntia4tzd4efvlfwcu4vjkpngnw4"
            .to_string(),
        plugin_address: "0x4838b106fce9647bdf1e7877bf73ce8b0bad5f97".to_string(),
        dao_address: "0x4838b106fce9647bdf1e7877bf73ce8b0bad5f97".to_string(),
    };
    let edit_events = vec![edit_event];

    sink.handle_space_created(&space_created_event, &edit_events, &block)
        .await?;
    println!("Space created");

    let time = std::time::Instant::now();
    sink.handle_edits_published(&edit_events, &[], &block)
        .await?;
    println!("Time taken: {:?}", time.elapsed());

    Ok(())
}
