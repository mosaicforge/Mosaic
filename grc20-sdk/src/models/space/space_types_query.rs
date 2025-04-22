use futures::{Stream, TryStreamExt};

use grc20_core::{
    error::DatabaseError,
    mapping::{
        entity_node, prop_filter,
        query_utils::{QueryStream, TypesFilter},
        EntityFilter, EntityNode, PropFilter, Query,
    },
    neo4rs, system_ids,
};

use super::ParentSpacesQuery;

/// Query to find all types defined in a space
pub struct FindSpaceTypeQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
    id: String,
    strict: bool,
}

impl FindSpaceTypeQuery {
    pub(crate) fn new(neo4j: neo4rs::Graph, space_id: String, id: String) -> Self {
        Self {
            neo4j,
            space_id,
            id,
            strict: true,
        }
    }

    /// Set whether to only query the given space or all parent spaces
    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }
}

impl Query<Option<EntityNode>> for FindSpaceTypeQuery {
    async fn send(self) -> Result<Option<EntityNode>, DatabaseError> {
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
        let mut results = entity_node::find_many(&self.neo4j)
            .with_filter(
                EntityFilter::default()
                    .id(prop_filter::value(self.id.clone()))
                    .relations(TypesFilter::default().r#type(system_ids::SCHEMA_TYPE))
                    .space_id(PropFilter::default().value_in(spaces)),
            )
            .limit(1)
            .send()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        // Check if the result is empty
        if let Some(entity) = results.pop() {
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }
}

/// Query to find all types defined in a space
pub struct FindSpaceTypesQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
    limit: usize,
    skip: Option<usize>,
    strict: bool,
}

impl FindSpaceTypesQuery {
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

impl QueryStream<EntityNode> for FindSpaceTypesQuery {
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
