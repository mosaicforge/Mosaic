use clap::{Args, Parser};
use futures::{stream, StreamExt};
use kg_node::ipfs::IpfsClient;
use kg_node::ops::conversions;
use kg_node::ops::ops::Op;
use kg_node::{bootstrap, kg};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const ROOT_SPACE_IMPORTS: &str = "bafkreif4acly7y46hx7optzfxtehxotizgqjz5h5vszo7vtmzsnm4ktxjy";
// const CONSTRUCTION_SPACE_IMPORTS: &str = "bafkreih3oxxoenvhrcb747ib6rh7gpnho2rzopdljrtiyafoesyxnrhziq";
const CONSTRUCTION_SPACE_IMPORTS: &str =
    "bafkreiadpdybqrlieaql57fjpcwhy25ut3s742qkhuxz4i6meahhrpvnf4";

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

    let bootstrap_ops = if args.rollup {
        conversions::batch_ops(bootstrap::bootstrap())
    } else {
        bootstrap::bootstrap().into_iter().map(Op::from).collect()
    };

    stream::iter(bootstrap_ops)
        .for_each_concurrent(1, |op| async {
            kg_client.handle_op(op).await.expect("Failed to handle op");
        })
        .await;

    let ipfs_client = IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/");

    // // Import root space ops
    let root_space_ops = ipfs_client.import_blob(ROOT_SPACE_IMPORTS).await?;

    let root_space_ops = if args.rollup {
        conversions::batch_ops(root_space_ops)
    } else {
        root_space_ops.into_iter().map(Op::from).collect()
    };

    stream::iter(root_space_ops)
        .for_each_concurrent(1, |op| async {
            kg_client.handle_op(op).await.expect("Failed to handle op");
        })
        .await;

    // // Import construction space ops
    // let construction_space_import = ipfs_client.import_blob(
    //     CONSTRUCTION_SPACE_IMPORTS,
    // ).await?;

    // // let construction_space_ops = conversions::batch_ops(construction_space_ops);
    // let construction_space_ops = construction_space_import.into_iter().map(Op::from);

    // stream::iter(construction_space_ops)
    //     .for_each_concurrent(1, |op| async {
    //         kg_client.handle_op(op).await.expect("Failed to handle op");
    //     })
    //     .await;

    Ok(())
}
