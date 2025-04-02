use futures::TryStreamExt;
use grc20_core::{
    error::DatabaseError, mapping::{triple, TriplesConversionError, Value}, neo4rs, system_ids
};

use crate::models::space::ParentSpacesQuery;

use super::space::SubspacesQuery;

#[grc20_core::entity]
#[grc20(schema_type = system_ids::ATTRIBUTE)]
pub struct Property {
    #[grc20(attribute = system_ids::AGGREGATION_DIRECTION)]
    aggregation_direction: AggregationDirection,
}

#[derive(Clone, Debug)]
pub enum AggregationDirection {
    Up,
    Down,
    Bidirectional,
}

impl From<AggregationDirection> for Value {
    fn from(direction: AggregationDirection) -> Self {
        match direction {
            AggregationDirection::Up => Value::text("Up"),
            AggregationDirection::Down => Value::text("Down"),
            AggregationDirection::Bidirectional => Value::text("Bidirectional"),
        }
    }
}

impl TryFrom<Value> for AggregationDirection {
    type Error = TriplesConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.value.as_str() {
            "Up" => Ok(AggregationDirection::Up),
            "Down" => Ok(AggregationDirection::Down),
            "Bidirectional" => Ok(AggregationDirection::Bidirectional),
            _ => Err(TriplesConversionError::InvalidValue(format!(
                "Invalid aggregation direction: {}",
                value.value
            ))),
        }
    }
}

async fn attribute_aggregation_direction(
    neo4j: &neo4rs::Graph,
    space_id: &str,
    attribute_id: &str,
) -> Result<Option<AggregationDirection>, DatabaseError> {
    // Hardcoded for now as the aggregation direction triples are not yet present
    // in the knowledge graph
    // Might be able to change this to actual queries later
    match attribute_id {
        // This is the "base case", unclear if it could be replaced with a query even
        // if present in the knowledge graph
        system_ids::AGGREGATION_DIRECTION => return Ok(Some(AggregationDirection::Down)),

        // These are hardcoded for now since they are not yet present in the knowledge graph
        system_ids::NAME_ATTRIBUTE => return Ok(Some(AggregationDirection::Down)),
        system_ids::DESCRIPTION_ATTRIBUTE => return Ok(Some(AggregationDirection::Down)),
        _ => (),
    }

    // Get all spaces to query (just the given space if strict, or all parent spaces if not)
    let mut spaces_to_query = vec![(space_id.to_string(), 0)];

    let parent_spaces = ParentSpacesQuery::new(neo4j.clone(), space_id.to_string())
        .max_depth(None)
        .send()
        .await?
        .try_collect::<Vec<_>>()
        .await?;

    spaces_to_query.extend(parent_spaces);

    // Note: This may not be necessary since the parent spaces are collected using DFS
    // (i.e. the parent spaces *should* be sorted by depth)
    spaces_to_query.sort_by_key(|(_, depth)| *depth);

    for (space_id, _) in spaces_to_query {
        let maybe_triple = triple::find_one(
            neo4j,
            system_ids::AGGREGATION_DIRECTION,
            attribute_id,
            space_id,
            None,
        )
        .send()
        .await?;

        if let Some(triple) = maybe_triple {
            let direction = AggregationDirection::try_from(triple.value)?;
            return Ok(Some(direction));
        }
    }

    Ok(None)
}

// TODO: Find a better place for this function
pub async fn get_triple(
    neo4j: &neo4rs::Graph,
    attribute_id: impl Into<String>,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
    strict: bool,
) -> Result<Option<triple::Triple>, DatabaseError> {
    let space_id = space_id.into();
    let entity_id = entity_id.into();
    let attribute_id = attribute_id.into();

    if strict {
        let maybe_triple = triple::find_one(
            neo4j,
            &attribute_id,
            &entity_id,
            &space_id,
            space_version.clone(),
        )
        .send()
        .await?;

        return Ok(maybe_triple);
    }

    match attribute_aggregation_direction(neo4j, &space_id, &attribute_id).await? {
        Some(AggregationDirection::Up) => {
            let mut subspaces = SubspacesQuery::new(neo4j.clone(), space_id.clone())
                .max_depth(None)
                .send()
                .await?
                .try_collect::<Vec<_>>()
                .await?;

            subspaces.sort_by_key(|(_, depth)| *depth);
            
            for (space_id, _) in subspaces {
                let maybe_triple = triple::find_one(
                    neo4j,
                    &attribute_id,
                    &entity_id,
                    &space_id,
                    space_version.clone(),
                )
                .send()
                .await?;

                if let Some(triple) = maybe_triple {
                    return Ok(Some(triple));
                }
            }

            Ok(None)
        }
        Some(AggregationDirection::Down) => {
            let mut parent_spaces = ParentSpacesQuery::new(neo4j.clone(), space_id.clone())
                .max_depth(None)
                .send()
                .await?
                .try_collect::<Vec<_>>()
                .await?;

            parent_spaces.sort_by_key(|(_, depth)| *depth);
            
            for (space_id, _) in parent_spaces {
                let maybe_triple = triple::find_one(
                    neo4j,
                    &attribute_id,
                    &entity_id,
                    &space_id,
                    space_version.clone(),
                )
                .send()
                .await?;

                if let Some(triple) = maybe_triple {
                    return Ok(Some(triple));
                }
            }

            Ok(None)
        }
        Some(AggregationDirection::Bidirectional) => {
            let subspaces = SubspacesQuery::new(neo4j.clone(), space_id.clone())
                .max_depth(None)
                .send()
                .await?
                .try_collect::<Vec<_>>()
                .await?;

            let parent_spaces = ParentSpacesQuery::new(neo4j.clone(), space_id.clone())
                .max_depth(None)
                .send()
                .await?
                .try_collect::<Vec<_>>()
                .await?;

            let mut spaces = subspaces.into_iter().chain(parent_spaces.into_iter()).collect::<Vec<_>>();
            spaces.sort_by_key(|(_, depth)| *depth);
            
            for (space_id, _) in spaces {
                let maybe_triple = triple::find_one(
                    neo4j,
                    &attribute_id,
                    &entity_id,
                    &space_id,
                    space_version.clone(),
                )
                .send()
                .await?;

                if let Some(triple) = maybe_triple {
                    return Ok(Some(triple));
                }
            }

            Ok(None)
        }
        None => {
            let maybe_triple = triple::find_one(
                neo4j,
                &attribute_id,
                &entity_id,
                &space_id,
                space_version,
            )
            .send()
            .await?;

            Ok(maybe_triple)
        },
    }
}
