use futures::{Stream, StreamExt, TryStreamExt};

use crate::{
    entity::utils::MatchEntity,
    error::DatabaseError,
    mapping::{
        query_utils::VersionFilter, AttributeNode, FromAttributes, PropFilter, QueryBuilder,
        QueryStream, Subquery,
    },
};

use super::{Entity, EntityFilter, EntityNode};

pub struct SearchFromRestrictions<T> {
    neo4j: neo4rs::Graph,
    vector: Vec<f64>,
    filter: EntityFilter,
    space_id: Option<PropFilter<String>>,
    version: VersionFilter,
    limit: usize,
    skip: Option<usize>,
    threshold: f64,

    _marker: std::marker::PhantomData<T>,
}

impl<T> SearchFromRestrictions<T> {
    pub fn new(neo4j: &neo4rs::Graph, vector: Vec<f64>) -> Self {
        Self {
            neo4j: neo4j.clone(),
            vector,
            filter: EntityFilter::default(),
            space_id: None,
            version: VersionFilter::default(),
            limit: 100,
            skip: None,
            threshold: 0.5,

            _marker: std::marker::PhantomData,
        }
    }

    pub fn filter(mut self, filter: EntityFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn space_id(mut self, filter: PropFilter<String>) -> Self {
        self.space_id = Some(filter);
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version.version_mut(version.into());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn limit_opt(mut self, limit: Option<usize>) -> Self {
        if let Some(limit) = limit {
            self.limit = limit;
        }
        self
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    pub fn skip_opt(mut self, skip: Option<usize>) -> Self {
        self.skip = skip;
        self
    }

    pub fn threshold(mut self, threshold: f64) -> Self {
        if (0.0..=1.0).contains(&threshold) {
            self.threshold = threshold
        }
        self
    }

    fn subquery(&self) -> QueryBuilder {
        const QUERY: &str = r#"
            CALL db.index.vector.queryNodes('vector_index', $limit * $effective_search_ratio, $vector)
            YIELD node AS n, score AS score
            WHERE score > $threshold
            MATCH (e:Entity) -[r:ATTRIBUTE]-> (n)
        "#;
        QueryBuilder::default()
            .subquery(QUERY)
            .subquery(self.filter.subquery("e"))
            .limit(self.limit)
            .skip_opt(self.skip)
            .params("vector", self.vector.clone())
            .params("effective_search_ratio", EFFECTIVE_SEARCH_RATIO)
            .params("limit", self.limit as i64)
            .params("threshold", self.threshold)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchFromRestrictionsResult<T> {
    pub entity: T,
}

const EFFECTIVE_SEARCH_RATIO: f64 = 100000.0; // Adjust this ratio based on your needs

impl QueryStream<SearchFromRestrictionsResult<EntityNode>> for SearchFromRestrictions<EntityNode> {
    async fn send(
        self,
    ) -> Result<
        impl Stream<Item = Result<SearchFromRestrictionsResult<EntityNode>, DatabaseError>>,
        DatabaseError,
    > {
        let query = self.subquery().r#return("DISTINCT e");

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!(
                "entity_node::SearchFromRestrictions::<EntityNode>:\n{}\nparams:{:?}",
                query.compile(),
                query.params()
            );
        };

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            e: EntityNode,
        }

        Ok(self
            .neo4j
            .execute(query.build())
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move { Ok(SearchFromRestrictionsResult { entity: row.e }) }))
    }
}

impl<T: FromAttributes> QueryStream<SearchFromRestrictionsResult<Entity<T>>>
    for SearchFromRestrictions<Entity<T>>
{
    async fn send(
        self,
    ) -> Result<
        impl Stream<Item = Result<SearchFromRestrictionsResult<Entity<T>>, DatabaseError>>,
        DatabaseError,
    > {
        let match_entity = MatchEntity::new(&self.space_id, &self.version);

        let query = self.subquery().with(
            vec!["e".to_string()],
            match_entity.chain(
                "e",
                "attrs",
                "types",
                Some(vec![]),
                "RETURN e{.*, attrs: attrs, types: types}",
            ),
        );

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!(
                "entity_node::SearchFromRestrictions::<Entity<T>>:\n{}\nparams:{:?}",
                query.compile(),
                query.params
            );
        };

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            node: EntityNode,
            attrs: Vec<AttributeNode>,
            types: Vec<EntityNode>,
        }

        let stream = self
            .neo4j
            .execute(query.build())
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .map(|row_result| {
                row_result.and_then(|row| {
                    T::from_attributes(row.attrs.into())
                        .map(|data| SearchFromRestrictionsResult {
                            entity: Entity {
                                node: row.node,
                                attributes: data,
                                types: row.types.into_iter().map(|t| t.id).collect(),
                            },
                        })
                        .map_err(DatabaseError::from)
                })
            });

        Ok(stream)
    }
}
