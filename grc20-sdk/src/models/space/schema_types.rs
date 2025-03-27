use std::collections::HashMap;

use futures::{pin_mut, StreamExt, TryStreamExt};

use grc20_core::{
    aggregation::AggregationError, mapping::{
        entity_node,
        query_utils::QueryStream,
    }, neo4rs
};

use super::{space_hierarchy, space_types_query::SpaceTypesQuery};

pub async fn space_schema_types(
    neo4j: &neo4rs::Graph,
    space_id: &str,
    strict: bool,
) -> Result<Vec<entity_node::EntityNode>, AggregationError> {
    // Initialize map to store unique types by ID
    let mut result_map = HashMap::new();

    // Get all spaces to query (just the given space if strict, or all parent spaces if not)
    let mut spaces_to_query = vec![(space_id.to_string(), 0)];
    if !strict {
        let parent_spaces = space_hierarchy::all_parent_spaces(neo4j, space_id).await?;
        spaces_to_query.extend(parent_spaces);
    }

    // Query types from each space
    for (space_id, _) in spaces_to_query {
        let stream = SpaceTypesQuery::new(neo4j.clone(), space_id)
            .send()
            .await?;

        pin_mut!(stream);

        // Add each type to the result map, keyed by ID
        while let Some(type_entity) = stream.try_next().await? {
            result_map.insert(type_entity.id.clone(), type_entity);
        }
    }

    // Convert the HashMap values to a Vec
    Ok(result_map.into_values().collect())
}
