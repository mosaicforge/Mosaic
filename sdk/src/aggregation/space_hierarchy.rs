use std::{collections::HashSet, pin::Pin};

use futures::{pin_mut, Stream, StreamExt, TryStreamExt};

use crate::{error::DatabaseError, mapping::query_utils::QueryStream, models::space::ParentSpacesQuery};

use super::error::AggregationError;

pub async fn all_parent_spaces(
    neo4rs: &neo4rs::Graph,
    space_id: &str,
) -> Result<Vec<String>, AggregationError> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = vec![space_id.to_string()];

    // Add initial space to visited set
    visited.insert(space_id.to_string());

    // Process spaces in queue until empty
    while let Some(current_space) = queue.pop() {
        // Get immediate parent spaces
        let parent_spaces = <ParentSpacesQuery as QueryStream<String>>::send(
                ParentSpacesQuery::new(neo4rs.clone(), current_space))
            .await?;

        pin_mut!(parent_spaces);

        while let Some(parent_space) = parent_spaces.try_next().await? {
            // Skip if we've already visited this space (handles cycles)
            if !visited.insert(parent_space.clone()) {
                continue;
            }

            // Add to result and queue for further processing
            result.push(parent_space.clone());
            queue.push(parent_space);
        }
    }

    Ok(result)
}
