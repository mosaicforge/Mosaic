use futures::{Stream, StreamExt};

use grc20_core::{
    entity::{self, EntityNodeRef}, error::DatabaseError, indexer_ids, mapping::{
        prop_filter, query_utils::QueryStream, Entity, PropFilter, Query, RelationEdge
    }, neo4rs, relation
};

use crate::models::Account;

/// Query to find all members of a space
pub struct SpaceMembersQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
    limit: usize,
    skip: Option<usize>,
}

impl SpaceMembersQuery {
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

impl QueryStream<Entity<Account>> for SpaceMembersQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Entity<Account>, DatabaseError>>, DatabaseError> {
        // Find all member relations for the space
        let relations_stream = relation::find_many::<RelationEdge<EntityNodeRef>>(&self.neo4j)
            .filter(
                relation::RelationFilter::default()
                    .to_(entity::EntityFilter::default().id(prop_filter::value(self.space_id)))
                    .relation_type(entity::EntityFilter::default().id(prop_filter::value(indexer_ids::MEMBER_RELATION))),
            )
            .space_id(PropFilter::default().value(indexer_ids::INDEXER_SPACE_ID))
            .limit(self.limit)
            .send()
            .await?;

        // Convert the stream of relations to a stream of accounts
        let neo4j = self.neo4j.clone();
        let account_stream = relations_stream
            .map(move |relation_result| {
                let neo4j = neo4j.clone();
                async move {
                    let relation = relation_result?;
                    entity::find_one::<Entity<Account>>(
                        &neo4j,
                        &relation.from,
                    )
                    .space_id(indexer_ids::INDEXER_SPACE_ID)
                    .send()
                    .await?
                    .ok_or_else(|| {
                        DatabaseError::NotFound(format!(
                            "Account with ID {} not found",
                            relation.from
                        ))
                    })
                }
            })
            .buffered(10); // Process up to 10 accounts concurrently

        Ok(account_stream)
    }
}
