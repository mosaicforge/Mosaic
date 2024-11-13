use clap::{Args, Parser};
use futures::{stream, StreamExt, TryStreamExt};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use ipfs::IpfsClient;
use kg_node::ops::conversions;
use kg_node::ops::ops::Op;
use kg_node::system_ids::ROOT_SPACE_ID;
use kg_node::{bootstrap, kg};
use kg_core::pb::grc20;

const ROOT_SPACE_IMPORTS: &str = "bafkreif4acly7y46hx7optzfxtehxotizgqjz5h5vszo7vtmzsnm4ktxjy";
// const CONSTRUCTION_SPACE_IMPORTS: &str = "bafkreih3oxxoenvhrcb747ib6rh7gpnho2rzopdljrtiyafoesyxnrhziq";
// const CONSTRUCTION_SPACE_IMPORTS: &str =
//     "bafkreiadpdybqrlieaql57fjpcwhy25ut3s742qkhuxz4i6meahhrpvnf4";
const CONSTRUCTION_SPACE_IMPORTS: &str = "bafkreidgyievktbezgsaxnnuylyn7acgy3kmaderzy4t4lwnfenhrggice";

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

    if args.reset_db {
        kg_client.reset_db(args.rollup).await?;
    }

    let kg_ref = &kg_client;


    let ipfs_client = IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/");

    // // Import root space ops
    let root_space_ops = import_space(&ipfs_client, ROOT_SPACE_IMPORTS).await?;

    let root_space_ops = if args.rollup {
        conversions::batch_ops(root_space_ops)
    } else {
        root_space_ops.into_iter().map(Op::from).collect()
    };

    stream::iter(root_space_ops)
        .map(Ok) // Convert to Result to be able to use try_for_each
        .try_for_each(|op| async move {
            op.apply_op(kg_ref, ROOT_SPACE_ID).await
        })
        .await?;

    // // Import construction space ops
    let construction_space_import = import_space(&ipfs_client,
        CONSTRUCTION_SPACE_IMPORTS,
    ).await?;

    let construction_space_ops = conversions::batch_ops(construction_space_import);
    // let construction_space_ops = construction_space_import.into_iter().map(Op::from);

    stream::iter(construction_space_ops)
        .map(Ok) // Convert to Result to be able to use try_for_each
        .try_for_each(|op| async move {
            op.apply_op(kg_ref, "0x74519E6EEc5BCFBC4Eb8F1A6d0C6D343173A286b").await
        })
        .await?;

    Ok(())
}

#[derive(Debug, Parser)]    
#[command(name = "stdout", version, about, arg_required_else_help = true)]
struct AppArgs {
    // #[clap(flatten)]
    // source_args: SourceArgs,
    #[clap(flatten)]
    neo4j_args: Neo4jArgs,
    // #[clap(subcommand)]
    // command: Command,
    /// Whether or not to roll up the relations
    #[arg(long)]
    rollup: bool,

    /// Whether or not to reset the database
    #[arg(long)]
    reset_db: bool,
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

async fn import_space(ipfs_client: &IpfsClient, ipfs_hash: &str) -> anyhow::Result<Vec<grc20::Op>> {
    let import = ipfs_client.get::<grc20::Import>(ipfs_hash, true).await?;

    Ok(stream::iter(import.edits)
        .then(|edit_hash| async move {
            let edit = ipfs_client.get::<grc20::ImportEdit>(&edit_hash, true).await?;
            anyhow::Ok(edit.ops)
        })
        .try_collect::<Vec<_>>()
        .await?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>())
}