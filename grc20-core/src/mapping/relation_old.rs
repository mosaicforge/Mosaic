use futures::{Stream, StreamExt, TryStreamExt};

use crate::{
    block::BlockMetadata,
    error::DatabaseError,
    indexer_ids,
    mapping::{AttributeNode, EntityNode},
};

use super::{
    attributes::IntoAttributes, entity_node::{self, EntityNodeRef}, prop_filter, query_utils::{query_part, Query, QueryPart, VersionFilter}, relation_edge, Entity, FromAttributes, PropFilter, QueryStream, RelationEdge, RelationFilter, Value
};

pub fn find_one<T>(
    neo4j: &neo4rs::Graph,
    id: impl Into<String>,
    space_id: impl Into<String>,
    version: Option<String>,
) -> FindOneQuery<T> {
    FindOneQuery::new(neo4j, id.into(), space_id.into(), version)
}

pub fn find_many<T>(neo4j: &neo4rs::Graph) -> FindManyQuery<T> {
    FindManyQuery::new(neo4j)
}

pub fn delete_one(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    relation_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> DeleteOneQuery {
    DeleteOneQuery::new(
        neo4j.clone(),
        block.clone(),
        relation_id.into(),
        space_id.into(),
        space_version.into(),
    )
}

pub struct FindManyQuery<T> {
    neo4j: neo4rs::Graph,
    id: Option<PropFilter<String>>,
    filter: RelationFilter,

    space_id: Option<PropFilter<String>>,
    version: VersionFilter,

    limit: usize,
    skip: Option<usize>,

    __phantom: std::marker::PhantomData<T>,
}

impl<T> FindManyQuery<T> {
    pub fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id: None,
            filter: RelationFilter::default(),
            space_id: None,
            version: VersionFilter::default(),
            limit: 100,
            skip: None,
            __phantom: std::marker::PhantomData,
        }
    }

    pub fn id(mut self, id: PropFilter<String>) -> Self {
        self.id = Some(id);
        self
    }

    pub fn filter(mut self, filter: RelationFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn space_id(mut self, space_id: PropFilter<String>) -> Self {
        self.space_id = Some(space_id);
        self
    }

    pub fn version(mut self, space_version: Option<String>) -> Self {
        if let Some(space_version) = space_version {
            self.version.version_mut(space_version);
        }
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    fn into_query_part(self) -> QueryPart {
        let mut query_part = QueryPart::default()
            .match_clause("(from:Entity) -[r:RELATION]-> (to:Entity)")
            .merge(self.filter.into_query_part("r", "from", "to"))
            .order_by_clause("r.index")
            .limit(self.limit);

        query_part = query_part
            .merge(self.version.clone().into_query_part("r"));

        if let Some(space_id) = &self.space_id {
            query_part = query_part
                .merge(space_id.clone().into_query_part("r", "space_id", None));
        }

        if let Some(skip) = self.skip {
            query_part = query_part.skip(skip);
        }

        query_part
    }

    pub fn select_to(self) -> FindManyToQuery {
        let mut query_part = QueryPart::default()
            .match_clause("(from:Entity) -[r:RELATION]-> (to:Entity)")
            .merge(self.filter.into_query_part("r", "from", "to"))
            .order_by_clause("r.index")
            .limit(self.limit);

            query_part = query_part
                .merge(self.version.clone().into_query_part("r"));

            if let Some(space_id) = &self.space_id {
                query_part = query_part
                    .merge(space_id.clone().into_query_part("r", "space_id", None));
            }

        if let Some(skip) = self.skip {
            query_part = query_part.skip(skip);
        }

        query_part = query_part.with_clause("DISTINCT to", {
            let mut query_part = QueryPart::default()
                .match_clause("(to) -[r:ATTRIBUTE]-> (n:Attribute)")
                .merge(self.version.clone().into_query_part("r"));

            if let Some(space_id) = &self.space_id {
                query_part.merge_mut(space_id.clone().into_query_part("r", "space_id", None));
            }
            query_part.with_clause(
                "to, collect(n{.*}) AS attrs",
                query_part::return_query("to{.*, attributes: attrs}"),
            )
        });

        FindManyToQuery {
            neo4j: self.neo4j.clone(),
            query_part,
        }
    }
}



pub struct FindManyToQuery {
    neo4j: neo4rs::Graph,
    query_part: QueryPart,
}

impl<T: FromAttributes> QueryStream<Entity<T>> for FindManyToQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Entity<T>, DatabaseError>>, DatabaseError> {
        let neo4j = self.neo4j.clone();

        let query = if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!("relation::FindManyToQuery:\n{}", self.query_part);
            self.query_part.build()
        } else {
            self.query_part.build()
        };

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            node: EntityNode,
            attributes: Vec<AttributeNode>,
        }

        let stream = neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .map(|row_result| {
                row_result.and_then(|row| {
                    T::from_attributes(row.attributes.into())
                        .map(|data| Entity {
                            node: row.node,
                            attributes: data,
                            types: vec![],
                        })
                        .map_err(DatabaseError::from)
                })
            });

        Ok(stream)
    }
}

pub struct DeleteOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    relation_id: String,
    space_id: String,
    space_version: String,
}

impl DeleteOneQuery {
    fn new(
        neo4j: neo4rs::Graph,
        block: BlockMetadata,
        relation_id: String,
        space_id: String,
        space_version: String,
    ) -> Self {
        DeleteOneQuery {
            neo4j,
            block,
            relation_id,
            space_id,
            space_version,
        }
    }
}

impl Query<()> for DeleteOneQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        entity_node::delete_one(
            &self.neo4j,
            &self.block,
            &self.relation_id,
            &self.space_id,
            &self.space_version,
        )
        .send()
        .await?;

        relation_edge::delete_one(
            &self.neo4j,
            &self.block,
            &self.relation_id,
            &self.space_id,
            &self.space_version,
        )
        .send()
        .await
    }
}


#[cfg(test)]
mod tests {
    use crate::{mapping::{self, triple, EntityFilter, Triple}, system_ids};

    use super::*;

    use futures::pin_mut;
    use testcontainers::{
        core::{IntoContainerPort, WaitFor},
        runners::AsyncRunner,
        GenericImage, ImageExt,
    };

    const BOLT_PORT: u16 = 7687;
    const HTTP_PORT: u16 = 7474;

    #[derive(Clone, Debug, PartialEq)]
    struct Foo {
        name: String,
        bar: u64,
    }

    impl mapping::IntoAttributes for Foo {
        fn into_attributes(self) -> Result<mapping::Attributes, mapping::TriplesConversionError> {
            Ok(mapping::Attributes::default()
                .attribute(("name", self.name))
                .attribute(("bar", self.bar)))
        }
    }

    impl mapping::FromAttributes for Foo {
        fn from_attributes(
            mut attributes: mapping::Attributes,
        ) -> Result<Self, mapping::TriplesConversionError> {
            Ok(Self {
                name: attributes.pop("name")?,
                bar: attributes.pop("bar")?,
            })
        }
    }




}
