use super::models::Entity;
use crate::mapping::value::models::Value;
use crate::{error::DatabaseError, mapping::EFFECTIVE_SEARCH_RATIO};
use futures::{Stream, TryStreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

/// Query struct for performing semantic search on entities using vector embeddings
#[derive(Clone)]
pub struct SemanticSearchQuery {
    neo4j: neo4rs::Graph,
    search_vector: Vec<f64>,
    limit: usize,
    skip: Option<usize>,
}

impl SemanticSearchQuery {
    /// Create a new SemanticSearchQuery instance
    pub fn new(neo4j: &neo4rs::Graph, search_vector: Vec<f64>) -> Self {
        Self {
            neo4j: neo4j.clone(),
            search_vector,
            limit: 100,
            skip: None,
        }
    }

    /// Set the limit for search results (builder pattern)
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set the skip offset for search results (builder pattern)
    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    /// Execute the semantic search query
    pub async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<SemanticSearchResult, DatabaseError>>, DatabaseError>
    {
        let query = "
            CALL db.index.vector.queryNodes('vector_index', $limit * $effective_search_ratio, $vector)
            YIELD node, score
            ORDER BY score DESC
            MATCH (e:Entity)-[r:PROPERTIES]->(node)
            WITH e, score
            OPTIONAL MATCH (e)-[p:PROPERTIES]->(props:Properties)
            WITH e, score, collect(p.space_id) AS spaces, collect(apoc.map.removeKey(properties(props), 'embedding')) AS props
            RETURN {entity_id: e.id, spaces: spaces, properties: props, score: score}
            LIMIT $limit
        ";

        let query = neo4rs::query(&query)
            .param("vector", self.search_vector)
            .param("limit", self.limit as i64)
            .param("effective_search_ratio", EFFECTIVE_SEARCH_RATIO);

        Ok(self
            .neo4j
            .execute(query)
            .await?
            .into_stream_as::<SemanticSearchResult>()
            .map_err(DatabaseError::from))
    }
}

/// Result struct representing a semantic search match with full entity data
#[derive(Clone, Debug, Deserialize)]
pub struct SemanticSearchResult {
    pub entity_id: Uuid,
    pub spaces: Vec<Uuid>,
    pub properties: Vec<HashMap<Uuid, String>>,
    pub score: f64,
}

impl SemanticSearchResult {
    /// Convert the search result into a full Entity with score
    pub fn into_entity_with_score(self) -> (Entity, f64) {
        let entity = Entity {
            id: self.entity_id,
            values: self
                .spaces
                .into_iter()
                .zip(self.properties)
                .map(|(space_id, props)| {
                    (
                        space_id,
                        props
                            .into_iter()
                            .map(|(key, value)| Value {
                                property: key,
                                value,
                                options: None,
                            })
                            .collect(),
                    )
                })
                .collect(),
        };
        (entity, self.score)
    }

    /// Get just the entity without the score
    pub fn into_entity(self) -> Entity {
        self.into_entity_with_score().0
    }
}

/// Create a new SemanticSearchQuery instance
pub fn search(neo4j: &neo4rs::Graph, search_vector: Vec<f64>) -> SemanticSearchQuery {
    SemanticSearchQuery::new(neo4j, search_vector)
}
