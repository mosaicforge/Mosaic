use crate::{error::DatabaseError, indexer_ids, mapping::{entity, Entity, Query}};

use super::Space;

/// Query to find a single space by ID
pub struct FindOneQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
}

impl FindOneQuery {
    pub(crate) fn new(neo4j: neo4rs::Graph, space_id: String) -> Self {
        Self { neo4j, space_id }
    }
}

impl Query<Option<Entity<Space>>> for FindOneQuery {
    async fn send(self) -> Result<Option<Entity<Space>>, DatabaseError> {
        entity::find_one(
            &self.neo4j,
            self.space_id,
            indexer_ids::INDEXER_SPACE_ID,
            None,
        )
        .send()
        .await
    }
}