use crate::{
    block::BlockMetadata,
    error::DatabaseError,
    ids,
    mapping::{attributes, IntoAttributes, Query, RelationEdge},
    relation, system_ids,
};

use super::Entity;

pub struct InsertOneQuery<T> {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    entity: T,
    space_id: String,
    space_version: String,
}

impl<T> InsertOneQuery<T> {
    pub(super) fn new(
        neo4j: neo4rs::Graph,
        block: BlockMetadata,
        entity: T,
        space_id: String,
        space_version: String,
    ) -> Self {
        InsertOneQuery {
            neo4j,
            block,
            entity,
            space_id,
            space_version,
        }
    }
}

impl<T: IntoAttributes> Query<()> for InsertOneQuery<Entity<T>> {
    async fn send(self) -> Result<(), DatabaseError> {
        // Insert the entity data
        attributes::insert_one(
            &self.neo4j,
            &self.block,
            &self.entity.node.id,
            &self.space_id,
            &self.space_version,
            self.entity.attributes,
        )
        .send()
        .await?;

        // Create the relations between the entity and its types
        let types_relations = self
            .entity
            .types
            .iter()
            .map(|t| {
                RelationEdge::new(
                    ids::create_id_from_unique_string(format!(
                        "{}:{}:{}:{}",
                        self.space_id,
                        self.entity.node.id,
                        system_ids::TYPES_ATTRIBUTE,
                        t,
                    )),
                    &self.entity.node.id,
                    t,
                    system_ids::TYPES_ATTRIBUTE,
                    "0",
                )
            })
            .collect::<Vec<_>>();

        // Insert the relations
        relation::insert_many(&self.neo4j, &self.block, &self.space_id, self.space_version)
            .relations(types_relations)
            .send()
            .await?;

        Ok(())
    }
}
