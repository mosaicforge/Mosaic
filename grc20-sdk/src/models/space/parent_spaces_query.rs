use std::collections::HashSet;

use async_stream::stream;
use futures::{pin_mut, Stream, StreamExt, TryStreamExt};

use grc20_core::{
    entity::{self, utils::MatchEntity, Entity, EntityNode},
    error::DatabaseError,
    indexer_ids,
    mapping::{aggregation::SpaceRanking, prop_filter, query_utils::{query_builder::{QueryBuilder, Subquery}, QueryStream, VersionFilter}, AttributeNode, EntityNodeRef, PropFilter, RelationEdge},
    neo4rs, relation,
};

use super::Space;

/// Query to find all parent spaces of a given space
pub struct ParentSpacesQuery<T> {
    neo4j: neo4rs::Graph,
    space_id: String,
    limit: usize,
    skip: Option<usize>,
    max_depth: Option<usize>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> ParentSpacesQuery<T> {
    pub(crate) fn new(neo4j: neo4rs::Graph, space_id: String) -> Self {
        Self {
            neo4j,
            space_id,
            limit: 100,
            skip: None,
            max_depth: Some(1),
            _marker: std::marker::PhantomData,
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

    /// Limit the depth of the search
    pub fn max_depth(mut self, max_depth: Option<usize>) -> Self {
        self.max_depth = max_depth;
        self
    }

    fn subquery(&self) -> QueryBuilder {
        QueryBuilder::default()
            .subquery(format!(
                r#"MATCH (start:Entity {{id: $space_id}}) (() -[r:RELATION {{relation_type: "{}", space_id: "{}"}}]-> (s:Entity)){{,}}"#,
                indexer_ids::PARENT_SPACE,
                indexer_ids::INDEXER_SPACE_ID,
            ))
            .subquery("WHERE size(s) = size(COLLECT { WITH s UNWIND s AS _ RETURN DISTINCT _ })")
            .subquery_opt(self.max_depth.map(|depth| format!("AND size(s) <= {}", depth)))
            .subquery("WITH {space_id: LAST([start] + s).id, depth: SIZE(s)} AS parent_spaces")
            .limit(self.limit)
            .skip_opt(self.skip)
            .params("space_id", self.space_id.clone())
    }
}

// impl QueryStream<Entity<Space>> for ParentSpacesQuery {
//     async fn send(
//         self,
//     ) -> Result<impl Stream<Item = Result<Entity<Space>, DatabaseError>>, DatabaseError> {
//         // Find all parent space relations for the space
//         let relations_stream = relation_node::find_many(&self.neo4j)
//             .relation_type(PropFilter::default().value(indexer_ids::PARENT_SPACE))
//             .from_id(PropFilter::default().value(self.space_id.clone()))
//             .space_id(PropFilter::default().value(indexer_ids::INDEXER_SPACE_ID))
//             .limit(self.limit)
//             .send()
//             .await?;

//         // Convert the stream of relations to a stream of spaces
//         let neo4j = self.neo4j.clone();
//         let space_stream = relations_stream
//             .map(move |relation_result| {
//                 let neo4j = neo4j.clone();
//                 async move {
//                     let relation = relation_result?;
//                     entity::find_one(&neo4j, &relation.to, indexer_ids::INDEXER_SPACE_ID, None)
//                         .send()
//                         .await?
//                         .ok_or_else(|| {
//                             DatabaseError::NotFound(format!(
//                                 "Space with ID {} not found",
//                                 relation.to
//                             ))
//                         })
//                 }
//             })
//             .buffered(10); // Process up to 10 spaces concurrently

//         Ok(space_stream)
//     }
// }

impl QueryStream<SpaceRanking> for ParentSpacesQuery<SpaceRanking> {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<SpaceRanking, DatabaseError>>, DatabaseError> {
        let query = self.subquery()
            .r#return("parent_spaces");

        Ok(self.neo4j
            .execute(query.build())
            .await?
            .into_stream_as::<SpaceRanking>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move { Ok(row) }))
    }
}

async fn immediate_parent_spaces(
    neo4j: &neo4rs::Graph,
    space_id: &str,
    limit: usize,
) -> Result<impl Stream<Item = Result<String, DatabaseError>>, DatabaseError> {
    // Find all parent space relations where this space is the parent
    let relations_stream = relation::find_many::<RelationEdge<EntityNodeRef>>(neo4j)
        .filter(
            relation::RelationFilter::default()
                .from_(entity::EntityFilter::default().id(prop_filter::value(space_id)))
                .relation_type(
                    entity::EntityFilter::default()
                        .id(prop_filter::value(indexer_ids::PARENT_SPACE)),
                ),
        )
        .space_id(PropFilter::default().value(indexer_ids::INDEXER_SPACE_ID))
        .limit(limit)
        .send()
        .await?;

    // Convert the stream of relations to a stream of spaces
    let space_stream = relations_stream
        .map(move |relation_result| relation_result.map(|relation| relation.to.into()));

    Ok(space_stream)
}
