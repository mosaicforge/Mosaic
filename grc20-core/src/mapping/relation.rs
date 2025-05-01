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

/// High level model encapsulating a relation and its attributes.
#[derive(Clone, Debug, PartialEq)]
pub struct Relation<T, N> {
    relation: RelationEdge<N>,

    pub attributes: T,
}

impl<T> Relation<T, EntityNodeRef> {
    pub fn new(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        relation_type: impl Into<String>,
        index: impl Into<Value>,
        attributes: T,
    ) -> Self {
        Relation {
            relation: RelationEdge::new(id, from, to, relation_type, index),
            attributes,
        }
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

pub struct FindOneQuery<T> {
    neo4j: neo4rs::Graph,
    id: String,
    space_id: String,
    version: VersionFilter,
    __phantom: std::marker::PhantomData<T>,
}

impl<T> FindOneQuery<T> {
    fn new(neo4j: &neo4rs::Graph, id: String, space_id: String, version: Option<String>) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id,
            space_id,
            version: VersionFilter::new(version),
            __phantom: std::marker::PhantomData,
        }
    }
}

impl<T: FromAttributes> Query<Option<Relation<T, EntityNodeRef>>> for FindOneQuery<EntityNodeRef> {
    async fn send(self) -> Result<Option<Relation<T, EntityNodeRef>>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query = QueryPart::default()
            .match_clause("(from:Entity) -[r:RELATION {id: $id, space_id: $space_id}]-> (to:Entity)")
            .merge(self.version.clone().into_query_part("r"))
            .order_by_clause("r.index")
            .with_clause("r, from, to", {
                QueryPart::default()
                    .match_clause("(r_e:Entity {id: r.id}) -[r_attr:ATTRIBUTE]-> (n:Attribute)")
                    .merge(prop_filter::value::<String>(self.space_id.clone()).into_query_part("r_attr", "space_id", None))
                    .merge(self.version.into_query_part("r_attr"))
                    .with_clause(
                        "r, r_e, from, to, collect(n{.*}) AS attrs",
                        query_part::return_query("r{.*, from: from.id, to: to.id, attributes: attrs} as r")
                    )
            })
            .params("id", self.id)
            .params("space_id", self.space_id)
            .build();

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            edge: RelationEdge<EntityNodeRef>,
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
                    relation: row.edge,
                    attributes: T::from_attributes(row.attributes.into())?,
                })
            })
            .transpose()
    }
}

impl<T: FromAttributes> Query<Option<Relation<T, EntityNode>>> for FindOneQuery<EntityNode> {
    async fn send(self) -> Result<Option<Relation<T, EntityNode>>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query = QueryPart::default()
            .match_clause("(from:Entity) -[r:RELATION {id: $id, space_id: $space_id}]-> (to:Entity)")
            .merge(self.version.clone().into_query_part("r"))
            .order_by_clause("r.index")
            .with_clause("r, from, to", {
                QueryPart::default()
                    .match_clause("(r_e:Entity {id: r.id}) -[r_attr:ATTRIBUTE]-> (n:Attribute)")
                    .merge(prop_filter::value::<String>(self.space_id.clone()).into_query_part("r_attr", "space_id", None))
                    .merge(self.version.into_query_part("r_attr"))
                    .with_clause(
                        "r, r_e, from, to, collect(n{.*}) AS attrs",
                        query_part::return_query("r{.*, from: from, to: to, attributes: attrs} as r")
                    )
            })
            .params("id", self.id)
            .params("space_id", self.space_id)
            .build();

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            edge: RelationEdge<EntityNode>,
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
                    relation: row.edge,
                    attributes: T::from_attributes(row.attributes.into())?,
                })
            })
            .transpose()
    }
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

impl<T: FromAttributes> QueryStream<Relation<T, EntityNodeRef>> for FindManyQuery<EntityNodeRef> {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Relation<T, EntityNodeRef>, DatabaseError>>, DatabaseError> {
        let neo4j = self.neo4j.clone();

        let version = self.version.clone();
        let space_id = self.space_id.clone();
        let query_part = self.into_query_part()
            .with_clause("r, from, to", {
                let mut query_part = QueryPart::default()
                    .match_clause("(r_e:Entity {id: r.id}) -[r_attr:ATTRIBUTE]-> (n:Attribute)")
                    .merge(version.clone().into_query_part("r_attr"));

                if let Some(space_id) = space_id {
                    query_part.merge_mut(space_id.clone().into_query_part("r_attr", "space_id", None));
                }
                query_part
                    .with_clause(
                        "r, r_e, from, to, collect(n{.*}) AS attrs",
                        query_part::return_query("r{.*, from: from.id, to: to.id, attributes: attrs} as r")
                    )
            });
        
        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!("relation_node::FindManyQuery:\n{}", query_part);
        };

        let query = query_part.build();

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            node: RelationEdge<EntityNodeRef>,
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
                        })
                        .map_err(DatabaseError::from)
                })
            });

        Ok(stream)
    }
}

impl<T: FromAttributes> QueryStream<Relation<T, EntityNode>> for FindManyQuery<EntityNode> {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Relation<T, EntityNode>, DatabaseError>>, DatabaseError> {
        let neo4j = self.neo4j.clone();

        let version = self.version.clone();
        let space_id = self.space_id.clone();
        let query_part = self.into_query_part()
            .with_clause("r, from, to", {
                let mut query_part = QueryPart::default()
                    .match_clause("(r_e:Entity {id: r.id}) -[r_attr:ATTRIBUTE]-> (n:Attribute)")
                    .merge(version.clone().into_query_part("r_attr"));

                if let Some(space_id) = space_id {
                    query_part.merge_mut(space_id.clone().into_query_part("r_attr", "space_id", None));
                }
                query_part
                    .with_clause(
                        "r, r_e, from, to, collect(n{.*}) AS attrs",
                        query_part::return_query("r{.*, from: from, to: to, attributes: attrs} as r")
                    )
            });
        
        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!("relation_node::FindManyQuery:\n{}", query_part);
        };

        let query = query_part.build();

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            node: RelationEdge<EntityNode>,
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

pub struct InsertOneQuery<T> {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    relation: Relation<T, EntityNodeRef>,
    space_id: String,
    space_version: String,
}

impl<T> InsertOneQuery<T> {
    fn new(
        neo4j: neo4rs::Graph,
        block: BlockMetadata,
        space_id: String,
        space_version: String,
        relation: Relation<T, EntityNodeRef>,
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
        // let rel_id = self.relation.relation.id.clone();

        // // Insert the relation node
        // relation_node::insert_one(
        //     &self.neo4j,
        //     &self.block,
        //     &self.space_id,
        //     &self.space_version,
        //     self.relation.relation,
        // )
        // .send()
        // .await?;

        // // Insert the relation attributes
        // attributes::insert_one(
        //     &self.neo4j,
        //     &self.block,
        //     rel_id,
        //     &self.space_id,
        //     &self.space_version,
        //     self.relation.attributes,
        // )
        // .send()
        // .await
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (from:Entity {{id: $relation.from}})
            MATCH (to:Entity {{id: $relation.to}})
            MERGE (from) -[r:RELATION]-> (to)
            ON CREATE SET r += {{
                id: $relation.id,
                space_id: $space_id,
                index: $relation.index,
                min_version: $space_version,
                relation_type: $relation.relation_type,
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number
            }}
            SET r += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            MERGE (e:Entity {{id: $relation.id}})
            WITH e
            UNWIND $attributes AS attribute
            CALL (e, attribute) {{
                MATCH (e) -[r:ATTRIBUTE {{space_id: $space_id}}]-> (:Attribute {{id: attribute.id}})
                WHERE r.max_version IS null AND r.min_version <> $space_version
                SET r.max_version = $space_version
            }}
            CALL (e, attribute) {{
                MERGE (e) -[:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (m:Attribute {{id: attribute.id}})
                SET m += attribute
            }}
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("relation", self.relation.relation)
            .param("attributes", self.relation.attributes.into_attributes()?)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
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

        let relation = Relation::new("rel_abc", "from_id", "to_id", "relation_type", 0u64, foo);

        relation
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let found_relation = find_one::<EntityNodeRef>(&neo4j, "rel_abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find relation")
            .expect("Relation not found");

        assert_eq!(found_relation, relation);
    }

    #[tokio::test]
    async fn test_insert_find_one_relation_node() {
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

        let relation = Relation::new("rel_abc", "from_id", "to_id", "relation_type", 0u64, foo.clone());

        relation
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let found_relation = find_one::<EntityNode>(&neo4j, "rel_abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find relation")
            .expect("Relation not found");

        assert_eq!(
            found_relation,
            Relation { 
                relation: RelationEdge {
                    id: "rel_abc".to_string(),
                    from: EntityNode {
                        id: "from_id".to_string(),
                        system_properties: BlockMetadata::default().into(),
                    },
                    to: EntityNode {
                        id: "to_id".to_string(),
                        system_properties: BlockMetadata::default().into(),
                    },
                    relation_type: "relation_type".to_string(),
                    index: "0".to_string(),
                    system_properties: BlockMetadata::default().into(),
                },
                attributes: foo, 
            },
        );
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

        let relation = Relation::new("rel_abc", "from_id", "to_id", "relation_type", 0u64, foo);

        relation
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let stream = find_many::<EntityNodeRef>(&neo4j)
            .space_id(prop_filter::value::<String>("ROOT"))
            .filter(
                RelationFilter::default()
                    .relation_type(EntityFilter::default().id(prop_filter::value("relation_type"))),
            )
            .limit(1)
            .send()
            .await
            .expect("Failed to find relations");

        pin_mut!(stream);

        let found_relation: Relation<Foo, EntityNodeRef> = stream
            .next()
            .await
            .expect("Failed to get next relation")
            .expect("Relation not found");

        assert_eq!(found_relation.relation.id, relation.relation.id);
        assert_eq!(found_relation.attributes, relation.attributes);
    }

    #[tokio::test]
    async fn test_insert_find_many_relations_node() {
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

        let relation = Relation::new("rel_abc", "from_id", "to_id", "relation_type", 0u64, foo);

        relation
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let stream = find_many::<EntityNode>(&neo4j)
            .space_id(prop_filter::value::<String>("ROOT"))
            .filter(
                RelationFilter::default()
                    .relation_type(EntityFilter::default().id(prop_filter::value("relation_type"))),
            )
            .limit(1)
            .send()
            .await
            .expect("Failed to find relations");

        pin_mut!(stream);

        let found_relation: Relation<Foo, EntityNode> = stream
            .next()
            .await
            .expect("Failed to get next relation")
            .expect("Relation not found");

        assert_eq!(found_relation.relation.id, relation.relation.id);
        assert_eq!(found_relation.attributes, relation.attributes);
    }
}
