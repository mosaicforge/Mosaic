use futures::Stream;

use crate::{error::DatabaseError, mapping::{entity_node::{self, EntityFilter}, query_utils::{QueryStream, TypesFilter}, EntityNode}, system_ids};

/// Query to find all types defined in a space
pub struct SpaceTypesQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
    limit: usize,
    skip: Option<usize>,
}

impl SpaceTypesQuery {
    pub(crate) fn new(neo4j: neo4rs::Graph, space_id: String) -> Self {
        Self {
            neo4j,
            space_id,
            limit: 100,
            skip: None,
        }
    }

    /// Limit the number of results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Skip a number of results
    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }
}

impl QueryStream<EntityNode> for SpaceTypesQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<EntityNode, DatabaseError>>, DatabaseError> {
        // Find all entities that have a TYPES relation to the Type entity
        let mut stream = entity_node::find_many(&self.neo4j)
            .with_filter(
                EntityFilter::default()
                    .relations(TypesFilter::default().r#type(system_ids::SCHEMA_TYPE))
                    .space_id(self.space_id),
            )
            .limit(self.limit);

        if let Some(skip) = self.skip {
            stream = stream.skip(skip);
        }

        stream.send().await
    }
}
