use super::models::Value;
use crate::{error::DatabaseError, mapping::EFFECTIVE_SEARCH_RATIO, system_ids};
use futures::{Stream, TryStreamExt};
use serde::Deserialize;
use uuid::Uuid;

/// Query struct for performing semantic search on values using vector embeddings
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
        // let skip_clause = match self.skip {
        //     Some(skip) => format!("SKIP {}", skip),
        //     None => String::new(),
        // };

        let query = format!("
            CALL db.index.vector.queryNodes('vector_index', $limit * $effective_search_ratio, $vector)
            YIELD node, score
            ORDER BY score DESC
            MATCH (e:Entity)-[r:PROPERTIES]->(node)
            RETURN {{value: {{property: \"{NAME_PROPERTY}\", value: node[\"{NAME_PROPERTY}\"]}}, entity: e.id, space_id: r.space_id, score: score}}
            LIMIT $limit
        ", NAME_PROPERTY = system_ids::NAME_PROPERTY);

        let query = neo4rs::query(&query)
            .param("vector", self.search_vector)
            .param("limit", self.limit as i64)
            .param("effective_search_ratio", EFFECTIVE_SEARCH_RATIO);

        // let mut result = self.neo4j.execute(query).await?;

        Ok(self
            .neo4j
            .execute(query)
            .await?
            .into_stream_as::<SemanticSearchResult>()
            .map_err(DatabaseError::from))
    }
}

/// Result struct representing a semantic search match
#[derive(Clone, Debug, Deserialize)]
pub struct SemanticSearchResult {
    pub entity: Uuid,
    pub value: Value,
    pub space_id: Uuid,
    pub score: f64,
}

/// Create a new SemanticSearchQuery instance
pub fn search(neo4j: &neo4rs::Graph, search_vector: Vec<f64>) -> SemanticSearchQuery {
    SemanticSearchQuery::new(neo4j, search_vector)
}
