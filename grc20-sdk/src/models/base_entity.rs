use futures::TryStreamExt;
use grc20_core::{
    entity::{Entity, EntityNodeRef},
    error::DatabaseError,
    mapping::{prop_filter, EntityFilter, RelationEdge},
    neo4rs,
    relation::{self, RelationFilter},
    system_ids,
};

#[grc20_core::entity]
pub struct BaseEntity {
    #[grc20(attribute = system_ids::NAME_ATTRIBUTE)]
    name: Option<String>,

    #[grc20(attribute = system_ids::DESCRIPTION_ATTRIBUTE)]
    description: Option<String>,
}

pub async fn blocks(
    neo4j: &neo4rs::Graph,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    version: Option<String>,
    _strict: bool,
) -> Result<Vec<Entity<BaseEntity>>, DatabaseError> {
    // TODO: Implement aggregation
    relation::find_many::<RelationEdge<EntityNodeRef>>(neo4j)
        .filter(
            RelationFilter::default()
                .from_(EntityFilter::default().id(prop_filter::value(entity_id.into())))
                .relation_type(EntityFilter::default().id(prop_filter::value(system_ids::BLOCKS))),
        )
        .space_id(prop_filter::value(space_id.into()))
        .version(version)
        .select_to::<Entity<BaseEntity>>()
        .send()
        .await?
        .try_collect::<Vec<_>>()
        .await
}

pub async fn types(
    neo4j: &neo4rs::Graph,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    version: Option<String>,
    _strict: bool,
) -> Result<Vec<Entity<BaseEntity>>, DatabaseError> {
    // TODO: Implement aggregation
    relation::find_many::<RelationEdge<EntityNodeRef>>(neo4j)
        .filter(
            RelationFilter::default()
                .from_(EntityFilter::default().id(prop_filter::value(entity_id.into())))
                .relation_type(
                    EntityFilter::default().id(prop_filter::value(system_ids::TYPES_ATTRIBUTE)),
                ),
        )
        .space_id(prop_filter::value(space_id.into()))
        .version(version)
        .select_to::<Entity<BaseEntity>>()
        .send()
        .await?
        .try_collect::<Vec<_>>()
        .await
}
