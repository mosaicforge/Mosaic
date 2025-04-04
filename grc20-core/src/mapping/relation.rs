use futures::{Stream, StreamExt, TryStreamExt};

use crate::{block::BlockMetadata, error::DatabaseError, mapping::{AttributeNode, EntityNode}, system_ids};

use super::{
    attributes::{self, IntoAttributes}, entity_node, prop_filter, query_utils::{query_part, Query, QueryPart, VersionFilter}, relation_node, Entity, EntityFilter, FromAttributes, PropFilter, QueryStream, RelationFilter, RelationNode, Value
};

/// High level model encapsulating a relation and its attributes.
#[derive(Clone, Debug, PartialEq)]
pub struct Relation<T> {
    relation: RelationNode,

    pub attributes: T,
    pub types: Vec<String>,
}

impl<T> Relation<T> {
    pub fn new(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        relation_type: impl Into<String>,
        index: impl Into<Value>,
        attributes: T,
    ) -> Self {
        Relation {
            relation: RelationNode::new(id, from, to, relation_type, index),
            attributes,
            types: vec![],
        }
    }

    pub fn with_type(mut self, r#type: String) -> Self {
        self.types.push(r#type);
        self
    }

    pub fn insert(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
    ) -> InsertOneQuery<T> {
        InsertOneQuery::new(
            neo4j.clone(),
            block.clone(),
            space_id.into(),
            space_version.into(),
            self,
        )
    }
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

pub struct FindOneQuery {
    neo4j: neo4rs::Graph,
    id: String,
    space_id: String,
    version: VersionFilter,
}

impl FindOneQuery {
    fn new(
        neo4j: &neo4rs::Graph,
        id: String,
        space_id: String,
        version: Option<String>,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id,
            space_id,
            version: VersionFilter::new(version),
        }
    }

    fn into_query_part(self) -> QueryPart {
        QueryPart::default()
            .match_clause("(e:Entity:Relation {id: $id})")
            .match_clause(format!(
                "(e) -[r_from:`{}` {{space_id: $space_id}}]-> (from:Entity)",
                system_ids::RELATION_FROM_ATTRIBUTE
            ))
            .match_clause(format!(
                "(e) -[r_to:`{}` {{space_id: $space_id}}]-> (to:Entity)",
                system_ids::RELATION_TO_ATTRIBUTE
            ))
            .match_clause(format!(
                "(e) -[r_rt:`{}` {{space_id: $space_id}}]-> (rt:Entity)",
                system_ids::RELATION_TYPE_ATTRIBUTE
            ))
            .match_clause(format!(
                r#"(e) -[r_index:ATTRIBUTE {{space_id: $space_id}}]-> (index:Attribute {{id: "{}"}})"#,
                system_ids::RELATION_INDEX
            ))
            .merge(self.version.clone().into_query_part("r_from"))
            .merge(self.version.clone().into_query_part("r_to"))
            .merge(self.version.clone().into_query_part("r_rt"))
            .merge(self.version.clone().into_query_part("r_index"))
            .order_by_clause("index.value")
            .with_clause("e, from, to, rt, index", {
                QueryPart::default()
                    .match_clause("(e) -[r:ATTRIBUTE]-> (n:Attribute)")
                    .merge(prop_filter::value::<String>(self.space_id.clone()).into_query_part("r", "space_id"))
                    .merge(self.version.into_query_part("r"))
                    .with_clause(
                        "e, from, to, rt, index, collect(n{.*}) AS attrs",
                        query_part::return_query("e{.*, from: from.id, to: to.id, relation_type: rt.id, index: index, attributes: attrs}")
                    )
            })
            .params("id", self.id)
            .params("space_id", self.space_id)
    }
}

impl<T: FromAttributes>  Query<Option<Relation<T>>> for FindOneQuery {
    async fn send(self) -> Result<Option<Relation<T>>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query = self.into_query_part().build();

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            node: RelationNode,
            attributes: Vec<AttributeNode>,
        }

        neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                Result::<_, DatabaseError>::Ok(Relation {
                    relation: row.node,
                    attributes: T::from_attributes(row.attributes.into())?,
                    types: vec![],
                })
            })
            .transpose()
    }
}

pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    id: Option<PropFilter<String>>,
    filter: RelationFilter,

    space_id: Option<PropFilter<String>>,
    version: VersionFilter,

    limit: usize,
    skip: Option<usize>,
}

impl FindManyQuery {
    pub fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id: None,
            filter: RelationFilter::default(),
            space_id: None,
            version: VersionFilter::default(),
            limit: 100,
            skip: None,
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
            .match_clause("(e:Entity:Relation)")
            .merge(self.filter.into_query_part("e"))
            .order_by_clause("index.value")
            .limit(self.limit);

        query_part = query_part
            .merge(self.version.clone().into_query_part("r_from"))
            .merge(self.version.clone().into_query_part("r_to"))
            .merge(self.version.clone().into_query_part("r_rt"))
            .merge(self.version.clone().into_query_part("r_index"));

        if let Some(space_id) = &self.space_id {
            query_part = query_part
                .merge(space_id.clone().into_query_part("r_from", "space_id"))
                .merge(space_id.clone().into_query_part("r_to", "space_id"))
                .merge(space_id.clone().into_query_part("r_rt", "space_id"))
                .merge(space_id.clone().into_query_part("r_index", "space_id"));
        }

        if let Some(skip) = self.skip {
            query_part = query_part.skip(skip);
        }

        query_part
            .with_clause("e, from, to, rt, index", {
                let mut query_part = QueryPart::default()
                    .match_clause("(e) -[r:ATTRIBUTE]-> (n:Attribute)")
                    .merge(self.version.clone().into_query_part("r"));

                if let Some(space_id) = &self.space_id {
                    query_part.merge_mut(space_id.clone().into_query_part("r", "space_id"));
                }
                query_part
                    .with_clause(
                        "e, from, to, rt, index, collect(n{.*}) AS attrs",
                        query_part::return_query("e{.*, from: from.id, to: to.id, relation_type: rt.id, index: index, attributes: attrs}")
                    )
            })
    }

    pub fn select_to(self) -> FindManyToQuery {
        let mut query_part = QueryPart::default()
            .match_clause("(e:Entity:Relation)")
            .merge(self.filter.into_query_part("e"))
            .order_by_clause("index.value")
            .limit(self.limit);

        query_part = query_part
            .merge(self.version.clone().into_query_part("r_from"))
            .merge(self.version.clone().into_query_part("r_to"))
            .merge(self.version.clone().into_query_part("r_rt"))
            .merge(self.version.clone().into_query_part("r_index"));

        if let Some(space_id) = &self.space_id {
            query_part = query_part
                .merge(space_id.clone().into_query_part("r_from", "space_id"))
                .merge(space_id.clone().into_query_part("r_to", "space_id"))
                .merge(space_id.clone().into_query_part("r_rt", "space_id"))
                .merge(space_id.clone().into_query_part("r_index", "space_id"));
        }

        if let Some(skip) = self.skip {
            query_part = query_part.skip(skip);
        }

        query_part = query_part
            .with_clause("DISTINCT to", {
                let mut query_part = QueryPart::default()
                    .match_clause("(to) -[r:ATTRIBUTE]-> (n:Attribute)")
                    .merge(self.version.clone().into_query_part("r"));

                if let Some(space_id) = &self.space_id {
                    query_part.merge_mut(space_id.clone().into_query_part("r", "space_id"));
                }
                query_part
                .with_clause(
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

impl<T: FromAttributes> QueryStream<Relation<T>> for FindManyQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Relation<T>, DatabaseError>>, DatabaseError> {
        let neo4j = self.neo4j.clone();

        let query = if cfg!(debug_assertions) || cfg!(test) {
            let query_part = self.into_query_part();
            tracing::info!("relation_node::FindManyQuery:\n{}", query_part);
            query_part.build()
        } else {
            self.into_query_part().build()
        };
        
        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            node: RelationNode,
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
                        .map(|attributes| Relation {
                            relation: row.node,
                            attributes,
                            types: vec![],
                        })
                        .map_err(DatabaseError::from)
                })
            });

        Ok(stream)
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

        relation_node::delete_one(
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

pub struct InsertOneQuery<T> {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    relation: Relation<T>,
    space_id: String,
    space_version: String,
}

impl<T> InsertOneQuery<T> {
    fn new(
        neo4j: neo4rs::Graph,
        block: BlockMetadata,
        space_id: String,
        space_version: String,
        relation: Relation<T>,
    ) -> Self {
        InsertOneQuery {
            neo4j,
            block,
            relation,
            space_id,
            space_version,
        }
    }
}

impl<T: IntoAttributes> Query<()> for InsertOneQuery<T> {
    async fn send(self) -> Result<(), DatabaseError> {
        let rel_id = self.relation.relation.id.clone();

        // Insert the relation node
        relation_node::insert_one(
            &self.neo4j,
            &self.block,
            &self.space_id,
            &self.space_version,
            self.relation.relation,
        )
        .send()
        .await?;

        // Insert the relation attributes
        attributes::insert_one(
            &self.neo4j,
            &self.block,
            rel_id,
            &self.space_id,
            &self.space_version,
            self.relation.attributes,
        )
        .send()
        .await
    }
}

#[cfg(test)]
mod tests {
    use crate::mapping::{self, triple, Triple};

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

    #[tokio::test]
    async fn test_insert_find_one_relation() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let container = GenericImage::new("neo4j", "2025.01.0-community")
            .with_wait_for(WaitFor::Duration {
                length: std::time::Duration::from_secs(5),
            })
            .with_exposed_port(BOLT_PORT.tcp())
            .with_exposed_port(HTTP_PORT.tcp())
            .with_env_var("NEO4J_AUTH", "none")
            .start()
            .await
            .expect("Failed to start Neo 4J container");

        let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
        let host = container.get_host().await.unwrap().to_string();

        let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
            .await
            .unwrap();

        let foo = Foo {
            name: "Alice".into(),
            bar: 42,
        };

        triple::insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .triples(vec![
                Triple::new("from_id", "name", "FooFrom"),
                Triple::new("to_id", "name", "FooTo"),
                Triple::new("relation_type", "name", "FooRelation"),
                Triple::new(system_ids::TYPES_ATTRIBUTE, "name", "Types"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation = Relation::new(
            "rel_abc",
            "from_id",
            "to_id",
            "relation_type",
            0u64,
            foo,
        );

        relation
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let found_relation = FindOneQuery::new(&neo4j, "rel_abc".into(), "ROOT".into(), None)
            .send()
            .await
            .expect("Failed to find relation")
            .expect("Relation not found");

        assert_eq!(found_relation, relation);
    }

    #[tokio::test]
    async fn test_insert_find_many_relations() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let container = GenericImage::new("neo4j", "2025.01.0-community")
            .with_wait_for(WaitFor::Duration {
                length: std::time::Duration::from_secs(5),
            })
            .with_exposed_port(BOLT_PORT.tcp())
            .with_exposed_port(HTTP_PORT.tcp())
            .with_env_var("NEO4J_AUTH", "none")
            .start()
            .await
            .expect("Failed to start Neo 4J container");

        let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
        let host = container.get_host().await.unwrap().to_string();

        let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
            .await
            .unwrap();

        let foo = Foo {
            name: "Alice".into(),
            bar: 42,
        };

        triple::insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .triples(vec![
                Triple::new("from_id", "name", "FooFrom"),
                Triple::new("to_id", "name", "FooTo"),
                Triple::new("relation_type", "name", "FooRelation"),
                Triple::new(system_ids::TYPES_ATTRIBUTE, "name", "Types"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation = Relation::new(
            "rel_abc",
            "from_id",
            "to_id",
            "relation_type",
            0u64,
            foo,
        );

        relation
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let stream = FindManyQuery::new(&neo4j)
            .space_id(prop_filter::value::<String>("ROOT"))
            .filter(RelationFilter::default()
                .relation_type(EntityFilter::default()
                    .id(prop_filter::value("relation_type"))
                )
            )
            .limit(1)
            .send()
            .await
            .expect("Failed to find relations");

        pin_mut!(stream);

        let found_relation: Relation<Foo> = stream
            .next()
            .await
            .expect("Failed to get next relation")
            .expect("Relation not found");

        assert_eq!(found_relation.relation.id, relation.relation.id);
        assert_eq!(found_relation.attributes, relation.attributes);
    }
}
