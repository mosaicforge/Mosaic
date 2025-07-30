use crate::entity::utils::EntityFilter;
use crate::entity::SemanticSearchResult;
use crate::mapping::query_utils::Subquery;
use crate::{
    error::DatabaseError,
    mapping::{query_utils::QueryBuilder, EFFECTIVE_SEARCH_RATIO},
};
use futures::{Stream, TryStreamExt};

/// Query struct for performing semantic search on entities using vector embeddings
#[derive(Clone)]
pub struct ExactSemanticSearchQuery {
    neo4j: neo4rs::Graph,
    search_vector: Vec<f64>,
    limit: usize,
    skip: Option<usize>,
    threshold: Option<f64>,
    filter: EntityFilter,
}

impl ExactSemanticSearchQuery {
    /// Create a new SemanticSearchQuery instance
    pub fn new(neo4j: &neo4rs::Graph, search_vector: Vec<f64>) -> Self {
        Self {
            neo4j: neo4j.clone(),
            search_vector,
            limit: 100,
            skip: None,
            threshold: None,
            filter: EntityFilter::default(),
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

    /// Set the threshold for minimum similarity score (builder pattern)
    pub fn threshold(mut self, threshold: f64) -> Self {
        self.threshold = Some(threshold);
        self
    }

    /// Set a filter to apply to the search results (builder pattern)
    pub fn filter(mut self, filter: impl Into<EntityFilter>) -> Self {
        self.filter = filter.into();
        self
    }

    /// Execute the semantic search query
    pub async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<SemanticSearchResult, DatabaseError>>, DatabaseError>
    {
        // Exact neighbor search using vector index (very expensive but allows prefiltering)
        // const QUERY: &str = const_format::formatcp!(
        //     r#"
        //     MATCH (e:Entity) -[r:ATTRIBUTE]-> (a:Attribute:Indexed)
        //     WHERE r.max_version IS null
        //     AND a.embedding IS NOT NULL
        //     WITH e, a, r, vector.similarity.cosine(a.embedding, $vector) AS score
        //     ORDER BY score DESC
        //     WHERE score IS NOT null
        //     LIMIT $limit
        //     RETURN a{{.*, entity: e.id, space_version: r.min_version, space_id: r.space_id, score: score}}
        //     "#,
        // );

        let builder = QueryBuilder::default()
            .subquery("MATCH (e:Entity)")
            .subquery(self.filter.subquery("e"))
            .subquery("WITH DISTINCT e")
            .subquery("MATCH (e)-[:PROPERTIES]->(props:Properties)")
            .subquery("WHERE props.embedding IS NOT NULL")
            .subquery("WITH e, vector.similarity.cosine(props.embedding, $vector) AS score")
            .subquery("WHERE score IS NOT NULL")
            .subquery("ORDER BY score DESC")
            .subquery("LIMIT $limit")
            .params("vector", self.search_vector)
            .params("limit", self.limit as i64)
            .params("effective_search_ratio", EFFECTIVE_SEARCH_RATIO)
            .params("threshold", self.threshold.unwrap_or(0.0));

        let query = builder
            .subquery(
                "OPTIONAL MATCH (e)-[p:PROPERTIES]->(props:Properties)
                WITH e, score, collect(p.space_id) AS spaces, collect(apoc.map.removeKey(properties(props), 'embedding')) AS props",
            )
            .skip_opt(self.skip)
            .limit(self.limit)
            .r#return("{entity_id: e.id, spaces: spaces, properties: props, score: score, types: labels(e)}")
            .build();

        Ok(self
            .neo4j
            .execute(query)
            .await?
            .into_stream_as::<SemanticSearchResult>()
            .map_err(DatabaseError::from))
    }
}

/// Create a new SemanticSearchQuery instance
pub fn exact_search(neo4j: &neo4rs::Graph, search_vector: Vec<f64>) -> ExactSemanticSearchQuery {
    ExactSemanticSearchQuery::new(neo4j, search_vector)
}
