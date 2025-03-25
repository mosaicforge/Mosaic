use crate::{error::DatabaseError, models::space};

#[derive(Debug, thiserror::Error)]
pub enum AggregationError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError)
}

/// Represents a type aggregation, i.e. a list of spaces that should be included in 
/// the aggregation for that type when querying the graph from the perspective of the
/// initial space.
#[derive(Debug, Clone)]
pub struct TypeAggregation {
    pub id: String,
    pub initial_space: String,
    pub spaces: Vec<String>,
}

/// Given a space id, returns a list of all spaces that should be included in the aggregation
pub async fn hierarchy_aggregation(
    neo4j: &neo4rs::Graph,
    space_id: &str,
) -> Result<Vec<TypeAggregation>, AggregationError> {
    // First, all types visible in the space
    // let types = space::types(space_id, neo4j).await?;

    todo!()
}