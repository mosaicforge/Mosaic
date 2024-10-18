use clap::{Args, Parser};
use futures::{stream, StreamExt};
use kg_node::ops::ops::Op;
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

    // let kg_client = kg_client::KgClient::new("neo4j://localhost:7687", "", "").await?;
    let kg_client = kg::Client::new(
        &args.neo4j_args.neo4j_uri,
        &args.neo4j_args.neo4j_user,
        &args.neo4j_args.neo4j_pass,
    )
    .await?;
    // kg_client.bootstrap().await?;

    // let ops = conversions::batch_ops(bootstrap::bootstrap());
    let ops = bootstrap::bootstrap().into_iter().map(Op::from);

    stream::iter(ops)
        .for_each_concurrent(1, |op| async {
            kg_client.handle_op(op).await.expect("Failed to handle op");
        })
        .await;

    // let buf = include_bytes!("../bafkreif4acly7y46hx7optzfxtehxotizgqjz5h5vszo7vtmzsnm4ktxjy");
    // let buf = include_bytes!("../bafkreih3oxxoenvhrcb747ib6rh7gpnho2rzopdljrtiyafoesyxnrhziq");
    // let ipfs_client = IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/");
    // import_blob(
    //     &kg_client,
    //     &ipfs_client,
    //     "bafkreif4acly7y46hx7optzfxtehxotizgqjz5h5vszo7vtmzsnm4ktxjy",
    // )
    // .await?;

    // import_blob(
    //     &kg_client,
    //     &ipfs_client,
    //     "bafkreih3oxxoenvhrcb747ib6rh7gpnho2rzopdljrtiyafoesyxnrhziq",
    // )
    // .await?;

    // let import = deserialize::<grc20::Import>(buf)?;
    // // // println!("{:?}", import);

    // let client = &IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/");

    // let mut edits = stream::iter(import.edits)
    //     .map(|edit| async move {
    //         let hash = edit.replace("ipfs://", "");
    //         let bytes = client.get(&hash).await?;
    //         anyhow::Ok(deserialize::<grc20::ImportEdit>(&bytes)?)
    //     })
    //     .buffer_unordered(10)
    //     .try_collect::<Vec<_>>()
    //     .await?;

    // edits.sort_by_key(|edit| edit.block_number.clone());

    // for edit in edits {
    //     // for op in edit.ops {
    //     //     kg_client.handle_op(op).await?;
    //     // }
    //     stream::iter(edit.ops)
    //         .for_each_concurrent(1, |op| async {
    //             kg_client.handle_op(op).await.expect("Failed to handle op");
    //         })
    //         .await;
    //     // break;
    // }

    Ok(())
}
