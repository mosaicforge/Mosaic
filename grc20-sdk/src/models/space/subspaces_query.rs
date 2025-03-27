use futures::{Stream, StreamExt};

use grc20_core::{error::DatabaseError, indexer_ids, mapping::{entity, query_utils::QueryStream, relation_node, Entity, PropFilter, Query}, neo4rs};

use super::Space;

/// Query to find all subspaces of a given space
pub struct SubspacesQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
    limit: usize,
    skip: Option<usize>,
}

impl SubspacesQuery {
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

impl QueryStream<Entity<Space>> for SubspacesQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Entity<Space>, DatabaseError>>, DatabaseError> {
        // Find all parent space relations where this space is the parent
        let relations_stream = relation_node::find_many(&self.neo4j)
            .relation_type(PropFilter::default().value(indexer_ids::PARENT_SPACE))
            .to_id(PropFilter::default().value(self.space_id.clone()))
            .space_id(PropFilter::default().value(indexer_ids::INDEXER_SPACE_ID))
            .limit(self.limit)
            .send()
            .await?;

        // Convert the stream of relations to a stream of spaces
        let neo4j = self.neo4j.clone();
        let space_stream = relations_stream
            .map(move |relation_result| {
                let neo4j = neo4j.clone();
                async move {
                    let relation = relation_result?;
                    entity::find_one(&neo4j, &relation.from, indexer_ids::INDEXER_SPACE_ID, None)
                        .send()
                        .await?
                        .ok_or_else(|| {
                            DatabaseError::NotFound(format!(
                                "Space with ID {} not found",
                                relation.from
                            ))
                        })
                }
            })
            .buffered(10); // Process up to 10 spaces concurrently

        Ok(space_stream)
    }
}

impl QueryStream<String> for SubspacesQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<String, DatabaseError>>, DatabaseError> {
        // Find all parent space relations where this space is the parent
        let relations_stream = relation_node::find_many(&self.neo4j)
            .relation_type(PropFilter::default().value(indexer_ids::PARENT_SPACE))
            .to_id(PropFilter::default().value(self.space_id.clone()))
            .space_id(PropFilter::default().value(indexer_ids::INDEXER_SPACE_ID))
            .limit(self.limit)
            .send()
            .await?;

        // Convert the stream of relations to a stream of spaces
        let space_stream = relations_stream
            .map(move |relation_result| {
                relation_result.map(|relation| relation.to)
            });

        Ok(space_stream)
    }
}
