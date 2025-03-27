use crate::mapping::{triple, Query, Triple};

use super::{error::AggregationError, space_hierarchy};

pub async fn get_triple(
    neo4j: &neo4rs::Graph,
    attribute_id: String,
    entity_id: String,
    space_id: String,
    space_version: Option<String>,
    strict: bool,
) -> Result<Option<Triple>, AggregationError> {
    // Get all spaces to query (just the given space if strict, or all parent spaces if not)
    let mut spaces_to_query = vec![(space_id.clone(), 0)];
    if !strict {
        let parent_spaces = space_hierarchy::all_parent_spaces(neo4j, &space_id).await?;
        spaces_to_query.extend(parent_spaces);
    }

    spaces_to_query.sort_by_key(|(_, depth)| *depth);

    for (space_id, _) in spaces_to_query {
        let maybe_triple = triple::find_one(
            neo4j,
            &attribute_id,
            &entity_id,
            space_id,
            space_version.clone(),
        )
        .send()
        .await?;

        if maybe_triple.is_some() {
            return Ok(maybe_triple);
        }
    }

    Ok(None)
}
