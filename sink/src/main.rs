use std::{env, sync::Arc};

use anyhow::Error;
use axum::{response::Json, routing::get, Router};
use cache::{CacheConfig, KgCache};
use std::time::Duration;
use clap::{Args, Parser};
use grc20_core::{
    block::BlockMetadata,
    indexer_ids,
    mapping::{self, query_utils::Query, triple},
    neo4rs,
};
use sink::bootstrap;
use sink::{events::EventHandler, metrics};
use substreams_utils::Sink;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const PKG_FILE: &str = "geo-substream.spkg";
const MODULE_NAME: &str = "geo_out";

const DEFAULT_START_BLOCK: u64 = 880;
const DEFAULT_END_BLOCK: u64 = 0;
const DEFAULT_HTTP_PORT: u16 = 8081;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let endpoint_url =
        env::var("SUBSTREAMS_ENDPOINT_URL").expect("SUBSTREAMS_ENDPOINT_URL not set");
    let start_block = env::var("SUBSTREAMS_START_BLOCK").unwrap_or_else(|_| {
        tracing::warn!(
            "SUBSTREAMS_START_BLOCK not set. Using default value: {}",
            DEFAULT_START_BLOCK
        );
        DEFAULT_START_BLOCK.to_string()
    });
    let end_block = env::var("SUBSTREAMS_END_BLOCK").unwrap_or_else(|_| {
        tracing::warn!(
            "SUBSTREAMS_END_BLOCK not set. Using default value: {}",
            DEFAULT_END_BLOCK
        );
        DEFAULT_END_BLOCK.to_string()
    });

    let args = AppArgs::parse();

    set_log_level();
    let _guard = init_tracing(args.log_file);

    let neo4j = neo4rs::Graph::new(
        &args.neo4j_args.neo4j_uri,
        &args.neo4j_args.neo4j_user,
        &args.neo4j_args.neo4j_pass,
    )
    .await?;

    if args.reset_db {
        reset_db(&neo4j).await?;
    } else {
        migration_check(&neo4j).await?;
    }

    let cache = if let Some(uri) = args.cache_args.memcache_uri {
        let cache_config = CacheConfig::new(vec![uri])
            .with_default_expiry(Duration::from_secs(args.cache_args.memcache_default_expiry));
        Some(Arc::new(KgCache::new(cache_config)?))
    } else {
        None
    };

    let sink = EventHandler::new(neo4j, cache)?;

    start_http_server().await;

    sink.run(
        &endpoint_url,
        PKG_FILE,
        MODULE_NAME,
        start_block
            .parse()
            .unwrap_or_else(|_| panic!("Invalid start block: {}! Must be integer", start_block)),
        end_block
            .parse()
            .unwrap_or_else(|_| panic!("Invalid end block: {}! Must be integer", end_block)),
        Some(16),
    )
    .await?;

    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "stdout", version, about, arg_required_else_help = true)]
struct AppArgs {
    #[clap(flatten)]
    neo4j_args: Neo4jArgs,

    #[clap(flatten)]
    cache_args: CacheArgs,

    /// Whether or not to roll up the relations
    #[arg(long, default_value_t = true)]
    rollup: bool,

    /// Whether or not to reset the database
    #[arg(long)]
    reset_db: bool,

    /// Log file path
    #[arg(long)]
    log_file: Option<String>,
}

#[derive(Debug, Args)]
struct Neo4jArgs {
    /// Neo4j database host
    #[arg(long)]
    neo4j_uri: String,

    /// Neo4j database user name
    #[arg(long)]
    neo4j_user: String,

    /// Neo4j database user password
    #[arg(long)]
    neo4j_pass: String,
}

#[derive(Debug, Args)]
struct CacheArgs {
    /// Memcache server URI (optional)
    #[arg(long, env = "memcache_uri")]
    memcache_uri: Option<String>,

    /// Default cache expiry in seconds
    #[arg(long, env = "memcache_default_expiry", default_value = "3600")]
    memcache_default_expiry: u64,
}

pub async fn reset_db(neo4j: &neo4rs::Graph) -> anyhow::Result<()> {
    // Delete all nodes and relations
    neo4j
        .run(neo4rs::query("MATCH (n) DETACH DELETE n"))
        .await?;

    // Delete indexes
    neo4j
        .run(neo4rs::query("DROP INDEX entity_id_index IF EXISTS"))
        .await?;
    neo4j
        .run(neo4rs::query("DROP INDEX relation_id_index IF EXISTS"))
        .await?;

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

    // Bootstrap indexer entities
    mapping::triple::insert_many(
        neo4j,
        &BlockMetadata::default(),
        indexer_ids::INDEXER_SPACE_ID,
        "0",
    )
    .triples(bootstrap::boostrap_indexer::triples())
    .send()
    .await?;

    Ok(())
}

async fn migration_check(neo4j: &neo4rs::Graph) -> Result<(), Error> {
    let version = triple::find_one(
        neo4j,
        indexer_ids::VERSION_ATTRIBUTE,
        indexer_ids::CURSOR_ID,
        indexer_ids::INDEXER_SPACE_ID,
        Some("0".to_string()),
    )
    .send()
    .await?;

    if let Some(version) = version {
        if version.value.value != env!("GIT_TAG") {
            tracing::info!(
                "Version mismatch. Resetting the database. Old version: {}, New version: {}",
                version.value.value,
                env!("GIT_TAG")
            );
            reset_db(neo4j).await?;
        } else {
            tracing::info!(
                "Version match: {}. No migration needed.",
                version.value.value
            );
        }
    } else {
        tracing::info!("No version found in the database. Resetting the database.");
        reset_db(neo4j).await?;
    }

    Ok(())
}

fn init_tracing(log_file: Option<String>) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    if let Some(log_file) = log_file {
        // Set the path of the log file
        let now = chrono::Utc::now();
        let file_appender = tracing_appender::rolling::never(
            ".",
            format!("{}-{log_file}", now.format("%Y-%m-%d-%H-%M-%S")),
        );
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::fmt()
            .with_writer(non_blocking)
            .with_ansi(false)
            .init();

        Some(_guard)
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "stdout=info".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();

        None
    }
}

fn set_log_level() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "component": "sink",
        "status": "ok",
    }))
}

async fn start_http_server() {
    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics::metrics_handler));

    let port = env::var("KG_SINK_HTTP_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or_else(|| {
            tracing::info!(
                "KG_SINK_HTTP_PORT not set, using default port {}",
                DEFAULT_HTTP_PORT
            );
            DEFAULT_HTTP_PORT
        });

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to start health server on {addr}: {e}"));
    tracing::info!("Health available on {addr}/health");
    tracing::info!("Metrics available on {addr}/metrics");

    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .unwrap_or_else(|e| panic!("failed to run HTTP server: {e}"));
    });
}
