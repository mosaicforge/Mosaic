use clap::{Args, Parser};
use futures::{stream, StreamExt};
use kg_node::ops::conversions;
use kg_node::{bootstrap, kg};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, Parser)]
#[command(name = "stdout", version, about, arg_required_else_help = true)]
struct AppArgs {
    // #[clap(flatten)]
    // source_args: SourceArgs,
    #[clap(flatten)]
    neo4j_args: Neo4jArgs,
    // #[clap(subcommand)]
    // command: Command,
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

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stdout=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn set_log_level() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    set_log_level();
    init_tracing();
    let args = AppArgs::parse();

    let kg_client = kg::Client::new(
        &args.neo4j_args.neo4j_uri,
        &args.neo4j_args.neo4j_user,
        &args.neo4j_args.neo4j_pass,
    )
    .await?;

    let ops = conversions::batch_ops(bootstrap::bootstrap());

    stream::iter(ops)
        .for_each_concurrent(1, |op| async {
            kg_client.handle_op(op).await.expect("Failed to handle op");
        })
        .await;

    Ok(())
}
