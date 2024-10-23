use clap::{Args, Parser};
use futures::{stream, StreamExt};
use kg_node::ipfs::IpfsClient;
use kg_node::ops::conversions;
use kg_node::ops::ops::Op;
use kg_node::{bootstrap, grc20, kg};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, Parser)]
#[command(name = "stdout", version, about, arg_required_else_help = true)]
struct AppArgs {
    /// The entity id to search triples for
    #[arg(long)]
    id: String,
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
    let args = &AppArgs::parse();

    let bootstrap_ops = bootstrap::bootstrap();

    stream::iter(bootstrap_ops)
        .for_each_concurrent(1, |op| async {
            match (op.r#type(), &op.triple) {
                (grc20::OpType::SetTriple, Some(grc20::Triple { entity, attribute, value: Some(grc20::Value {value, ..}) })) 
                if *entity == args.id || *attribute == args.id || *value == args.id => {
                    println!("SetTriple: {:?}", op.triple.unwrap());
                }
                (grc20::OpType::DeleteTriple, Some(grc20::Triple { entity, attribute, value: Some(grc20::Value {value, ..}) })) 
                if *entity == args.id || *attribute == args.id || *value == args.id => {
                    println!("DeleteTriple: {:?}", op.triple.unwrap());
                }
                _ => ()
            }
        })
        .await;

    let ipfs_client = IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/");

    // // Import root space ops 
    let root_space_ops = ipfs_client.import_blob(
        "bafkreif4acly7y46hx7optzfxtehxotizgqjz5h5vszo7vtmzsnm4ktxjy",
    )
    .await?;
    
    stream::iter(root_space_ops)
        .for_each_concurrent(1, |op| async {
            match (op.r#type(), &op.triple) {
                (grc20::OpType::SetTriple, Some(grc20::Triple { entity, attribute, value: Some(grc20::Value {value, ..}) })) 
                if *entity == args.id || *attribute == args.id || *value == args.id => {
                    println!("SetTriple: {:?}", op.triple.unwrap());
                }
                (grc20::OpType::DeleteTriple, Some(grc20::Triple { entity, attribute, value: Some(grc20::Value {value, ..}) })) 
                if *entity == args.id || *attribute == args.id || *value == args.id => {
                    println!("DeleteTriple: {:?}", op.triple.unwrap());
                }
                _ => ()
            }
        })
        .await;

    Ok(())
}