use futures::{pin_mut, StreamExt};
use grc20_core::{mapping::{aggregation::SpaceRanking, query_utils::QueryStream}, neo4rs};
use grc20_sdk::models::space;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get space ID from command line args
    let space_id = std::env::args()
        .nth(1)
        .expect("Please provide a space ID as the first argument");

    // Initialize Neo4j connection
    let neo4j = neo4rs::Graph::new("bolt://localhost:7687", "neo4j", "password").await?;

    // Create and execute subspaces query using Space helper
    let query = space::subspaces(&neo4j, &space_id)
        .max_depth(Some(10)) // Get all subspaces at any depth
        .limit(100); // Limit to 100 results

    let stream = query.send().await?;
    pin_mut!(stream);

    // Print each subspace ID as we receive it
    println!("Found subspaces:");
    while let Some(result) = stream.next().await {
        match result {
            Ok(SpaceRanking {space_id, ..}) => println!("  {}", space_id),
            Err(e) => eprintln!("Error getting subspace: {}", e),
        }
    }

    Ok(())
}
