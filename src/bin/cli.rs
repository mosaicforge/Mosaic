use clap::{Args, Parser, Subcommand};
use futures::{stream, StreamExt};
use kg_node::ipfs::IpfsClient;
use kg_node::kg::grc20::{Entity, EntityNode};
use kg_node::ops::conversions;
use kg_node::{bootstrap, grc20, kg, system_ids};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, Parser)]
#[command(name = "stdout", version, about, arg_required_else_help = true)]
struct AppArgs {
    // #[clap(flatten)]
    // source_args: SourceArgs,
    #[clap(flatten)]
    neo4j_args: Neo4jArgs,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Find triples related to an entity
    FindTriples {
        /// Entity ID
        id: String,
    },

    /// Describe an entity
    Describe {
        /// Entity ID
        id: String,
    },
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

pub fn find_triples(
    ops: impl IntoIterator<Item = grc20::Op>,
    entity_id: &str,
) -> Vec<(grc20::OpType, grc20::Triple)> {
    ops.into_iter()
        .filter_map(|op| match (op.r#type(), &op.triple) {
            (
                grc20::OpType::SetTriple,
                Some(grc20::Triple {
                    entity,
                    attribute,
                    value: Some(grc20::Value { value, .. }),
                }),
            ) if *entity == entity_id || *attribute == entity_id || *value == entity_id => Some((
                grc20::OpType::SetTriple,
                op.triple.expect("Triple should be some"),
            )),

            (
                grc20::OpType::DeleteTriple,
                Some(grc20::Triple {
                    entity,
                    attribute,
                    value: Some(grc20::Value { value, .. }),
                }),
            ) if *entity == entity_id || *attribute == entity_id || *value == entity_id => Some((
                grc20::OpType::DeleteTriple,
                op.triple.expect("Triple should be some"),
            )),
            _ => None,
        })
        .collect()
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
    let ipfs_client = IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/");

    let bootstrap_ops = bootstrap::bootstrap();

    // // Import root space ops
    let root_space_ops = ipfs_client
        .import_blob("bafkreif4acly7y46hx7optzfxtehxotizgqjz5h5vszo7vtmzsnm4ktxjy")
        .await?;

    match args.command {
        Command::FindTriples { id } => {
            let ops = find_triples(bootstrap_ops, &id)
                .into_iter()
                .chain(find_triples(root_space_ops, &id));

            for (op_type, triple) in ops {
                match op_type {
                    grc20::OpType::SetTriple => println!("SetTriple: {:?}", triple),
                    grc20::OpType::DeleteTriple => println!("DeleteTriple: {:?}", triple),
                    _ => (),
                }
            }
        }
        Command::Describe { id } => {
            let entity_node = kg_client
                .find_node_by_id::<EntityNode>(&id)
                .await?
                .expect("Entity not found");

            let entity = Entity::from_entity(kg_client.clone(), entity_node);

            println!("Entity: {}", entity);

            let attributes = entity.attributes().await?;

            for attribute in attributes {
                println!("\tAttribute: {}", attribute);
                if let Some(value_type) = attribute.value_type().await? {
                    println!("\t\tValue type: {}", value_type);
                }
            }
        }
    }

    Ok(())
}
