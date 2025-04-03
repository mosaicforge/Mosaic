use futures::{Stream, TryStreamExt};

use grc20_core::{
    error::DatabaseError,
    mapping::{
        EntityFilter,
        entity_node,
        query_utils::{QueryStream, TypesFilter},
        EntityNode, PropFilter,
    },
    neo4rs, system_ids,
};

use super::ParentSpacesQuery;

/// Query to find all types defined in a space
pub struct SpaceTypesQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
    limit: usize,
    skip: Option<usize>,
    strict: bool,
}

impl SpaceTypesQuery {
    pub(crate) fn new(neo4j: neo4rs::Graph, space_id: String) -> Self {
        Self {
            neo4j,
            space_id,
            limit: 100,
            skip: None,
            strict: true,
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

    /// Set whether to only query the given space or all parent spaces
    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }
}

impl QueryStream<EntityNode> for SpaceTypesQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<EntityNode, DatabaseError>>, DatabaseError> {
        let mut spaces = vec![self.space_id.clone()];

        if !self.strict {
            let parent_spaces: Vec<String> =
                ParentSpacesQuery::new(self.neo4j.clone(), self.space_id.clone())
                    .max_depth(None)
                    .send()
                    .await?
                    .map_ok(|(space, _)| space)
                    .try_collect()
                    .await?;

            spaces.extend(parent_spaces);
        }

        // Find all entities that have a TYPES relation to the Type entity
        let mut query = entity_node::find_many(&self.neo4j)
            .with_filter(
                EntityFilter::default()
                    .relations(TypesFilter::default().r#type(system_ids::SCHEMA_TYPE))
                    .space_id(PropFilter::default().value_in(spaces)),
            )
            .limit(self.limit);

        if let Some(skip) = self.skip {
            query = query.skip(skip);
        }

        query.send().await
    }
}
