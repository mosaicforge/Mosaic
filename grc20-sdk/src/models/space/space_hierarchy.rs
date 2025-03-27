use std::collections::HashSet;

use futures::{pin_mut, TryStreamExt};

use grc20_core::{aggregation::AggregationError, mapping::query_utils::QueryStream, neo4rs};

use crate::models::space::{ParentSpacesQuery, SubspacesQuery};

pub async fn all_parent_spaces(
    neo4rs: &neo4rs::Graph,
    space_id: &str,
) -> Result<Vec<(String, u64)>, AggregationError> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = vec![(space_id.to_string(), 0)]; // Include depth in the queue

    // Add initial space to visited set
    visited.insert(space_id.to_string());

    // Process spaces in queue until empty
    while let Some((current_space, depth)) = queue.pop() {
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
            result.push((parent_space.clone(), depth + 1));
            queue.push((parent_space, depth + 1));
        }
    }

    Ok(result)
}

pub async fn all_subspaces(
    neo4rs: &neo4rs::Graph,
    space_id: &str,
) -> Result<Vec<(String, u64)>, AggregationError> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = vec![(space_id.to_string(), 0)]; // Include depth in the queue

    // Add initial space to visited set
    visited.insert(space_id.to_string());

    // Process spaces in queue until empty
    while let Some((current_space, depth)) = queue.pop() {
        // Get immediate subspaces
        let subspaces = <SubspacesQuery as QueryStream<String>>::send(
                SubspacesQuery::new(neo4rs.clone(), current_space))
            .await?;

        pin_mut!(subspaces);

        while let Some(subspace) = subspaces.try_next().await? {
            // Skip if we've already visited this space (handles cycles)
            if !visited.insert(subspace.clone()) {
                continue;
            }

            // Add to result and queue for further processing
            result.push((subspace.clone(), depth + 1));
            queue.push((subspace, depth + 1));
        }
    }

    Ok(result)
}