use futures::{pin_mut, StreamExt, TryStreamExt};
use grc20_core::{mapping::query_utils::QueryStream, neo4rs, system_ids};
use grc20_sdk::models::{property, space};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stdout=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Get space ID from command line args
    let space_id = std::env::args()
        .nth(1)
        .expect("Please provide a space ID as the first argument");

    // Initialize Neo4j connection
    let neo4j = neo4rs::Graph::new("bolt://localhost:7687", "neo4j", "password").await?;

    let types = space::types(&neo4j, &space_id).strict(false).send().await?;

    pin_mut!(types);

    while let Some(type_) = types.next().await {
        match type_ {
            Ok(type_) => {
                let name = property::get_triple(
                    &neo4j,
                    system_ids::NAME_ATTRIBUTE,
                    &type_.id,
                    &space_id,
                    None,
                    false,
                )
                .await?
                .map(|triple| triple.value.value);

                println!(
                    "Type: {} ({})",
                    name.unwrap_or("null".to_string()),
                    type_.id
                );

                let properties = property::get_outbound_relations(
                    &neo4j,
                    system_ids::PROPERTIES,
                    type_.id,
                    &space_id,
                    None,
                    None,
                    None,
                    false,
                )
                .await?;

                pin_mut!(properties);

                while let Some(property) = properties.next().await {
                    match property {
                        Ok(property) => {
                            let name = property::get_triple(
                                &neo4j,
                                system_ids::NAME_ATTRIBUTE,
                                &property.to,
                                &space_id,
                                None,
                                false,
                            )
                            .await?
                            .map(|triple| triple.value.value);

                            let value_type = property::get_outbound_relations(
                                &neo4j,
                                system_ids::VALUE_TYPE_ATTRIBUTE,
                                &property.to,
                                &space_id,
                                None,
                                Some(1),
                                None,
                                false,
                            )
                            .await?
                            .try_collect::<Vec<_>>()
                            .await?;

                            let value_type_name = if let Some(value_type) = value_type.first() {
                                property::get_triple(
                                    &neo4j,
                                    system_ids::NAME_ATTRIBUTE,
                                    &value_type.to,
                                    &space_id,
                                    None,
                                    false,
                                )
                                .await?
                                .map(|triple| triple.value.value)
                            } else {
                                None
                            };

                            println!(
                                "  Property: {}: {} ({})",
                                name.unwrap_or("null".to_string()),
                                value_type_name.unwrap_or("null".to_string()),
                                property.to
                            );
                        }
                        Err(e) => eprintln!("Error: {:?}", e),
                    }
                }
            }
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    Ok(())
}
