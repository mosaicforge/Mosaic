use std::env;

use grc20_core::{
    block::BlockMetadata,
    mapping::{triple, Query},
    neo4rs,
};
use sink::events::EventHandler;
use substreams_utils::sink::Sink;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    GenericImage, ImageExt,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const BOLT_PORT: u16 = 7687;
const HTTP_PORT: u16 = 7474;

const PKG_FILE: &str = "geo-substream.spkg";
const MODULE_NAME: &str = "geo_out";

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

    // Setup tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stdout=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

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

    // Bootstrap the database
    let block = BlockMetadata::default();
    triple::insert_many(&neo4j, &block, "ROOT", "0")
        .triples(sink::bootstrap::boostrap_indexer::triples())
        .send()
        .await
        .expect("Failed to bootstrap indexer");

    println!("Neo4J database reset");

    let sink = EventHandler::new(neo4j, None)?;

    let endpoint_url =
        env::var("SUBSTREAMS_ENDPOINT_URL").expect("SUBSTREAMS_ENDPOINT_URL not set");

    sink.run(&endpoint_url, PKG_FILE, MODULE_NAME, 515, 1000, Some(32))
        .await?;

    Ok(())
}
