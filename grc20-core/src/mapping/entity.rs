use futures::{Stream, StreamExt, TryStreamExt};

use crate::{block::BlockMetadata, error::DatabaseError, ids, mapping::AttributeNode, system_ids};

use super::{
    attributes::{self, FromAttributes, IntoAttributes}, entity_node::SystemProperties, order_by::FieldOrderBy, prop_filter, query_utils::{
        query_part, AttributeFilter, PropFilter, Query, QueryPart, QueryStream, VersionFilter,
    }, relation_node, EntityFilter, EntityNode, RelationNode
};

/// High level model encapsulating an entity and its attributes.
#[derive(Clone, Debug, PartialEq)]
pub struct Entity<T> {
    pub(crate) node: EntityNode,
    pub attributes: T,
    pub types: Vec<String>,
}

impl<T> Entity<T> {
    pub fn new(id: impl Into<String>, attributes: T) -> Self {
        Entity {
            node: EntityNode {
                id: id.into(),
                system_properties: SystemProperties::default(),
            },
            attributes,
            types: vec![],
        }
    }

    pub fn id(&self) -> &str {
        &self.node.id
    }

    pub fn system_properties(&self) -> &SystemProperties {
        &self.node.system_properties
    }

    pub fn with_type(mut self, r#type: impl Into<String>) -> Self {
        self.types.push(r#type.into());
        self
    }

    pub fn with_types(mut self, types: impl IntoIterator<Item = String>) -> Self {
        self.types.extend(types);
        self
    }

    pub fn get_outbound_relations(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: impl Into<String>,
        space_version: Option<String>,
    ) -> relation_node::FindManyQuery {
        relation_node::FindManyQuery::new(neo4j)
            .from_id(prop_filter::value(&self.node.id))
            .space_id(prop_filter::value(space_id.into()))
            .version(space_version)
    }

    pub fn get_inbound_relations(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: impl Into<String>,
        space_version: Option<String>,
    ) -> relation_node::FindManyQuery {
        relation_node::FindManyQuery::new(neo4j)
            .to_id(prop_filter::value(&self.node.id))
            .space_id(prop_filter::value(space_id.into()))
            .version(space_version)
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
            self,
            space_id.into(),
            space_version.into(),
        )
    }
}

pub fn find_one(
    neo4j: &neo4rs::Graph,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
) -> FindOneQuery {
    FindOneQuery::new(
        neo4j.clone(),
        entity_id.into(),
        space_id.into(),
        space_version,
    )
}

pub fn find_many(
    neo4j: &neo4rs::Graph,
    space_id: impl Into<String>,
    space_version: Option<String>,
) -> FindManyQuery {
    FindManyQuery::new(neo4j.clone(), space_id.into(), space_version)
}

pub struct InsertOneQuery<T> {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    entity: Entity<T>,
    space_id: String,
    space_version: String,
}

impl<T> InsertOneQuery<T> {
    fn new(
        neo4j: neo4rs::Graph,
        block: BlockMetadata,
        entity: Entity<T>,
        space_id: String,
        space_version: String,
    ) -> Self {
        InsertOneQuery {
            neo4j,
            block,
            entity,
            space_id,
            space_version,
        }
    }
}

impl<T: IntoAttributes> Query<()> for InsertOneQuery<T> {
    async fn send(self) -> Result<(), DatabaseError> {
        // Insert the entity data
        attributes::insert_one(
            &self.neo4j,
            &self.block,
            &self.entity.node.id,
            &self.space_id,
            &self.space_version,
            self.entity.attributes,
        )
        .send()
        .await?;

        // Create the relations between the entity and its types
        let types_relations = self
            .entity
            .types
            .iter()
            .map(|t| {
                RelationNode::new(
                    ids::create_id_from_unique_string(format!(
                        "{}:{}:{}:{}",
                        self.space_id,
                        self.entity.node.id,
                        system_ids::TYPES_ATTRIBUTE,
                        t,
                    )),
                    &self.entity.node.id,
                    t,
                    system_ids::TYPES_ATTRIBUTE,
                    "0",
                )
            })
            .collect::<Vec<_>>();

        // Insert the relations
        relation_node::insert_many(&self.neo4j, &self.block, &self.space_id, self.space_version)
            .relations(types_relations)
            .send()
            .await?;

        Ok(())
    }
}

pub struct FindOneQuery {
    neo4j: neo4rs::Graph,
    entity_id: String,
    space_id: String,
    space_version: Option<String>,
}

impl FindOneQuery {
    fn new(
        neo4j: neo4rs::Graph,
        entity_id: String,
        space_id: String,
        space_version: Option<String>,
    ) -> Self {
        FindOneQuery {
            neo4j,
            entity_id,
            space_id,
            space_version,
        }
    }
}

impl<T: FromAttributes> Query<Option<Entity<T>>> for FindOneQuery {
    async fn send(self) -> Result<Option<Entity<T>>, DatabaseError> {
        let attributes = attributes::find_one(
            &self.neo4j,
            &self.entity_id,
            &self.space_id,
            self.space_version,
        )
        .send()
        .await?;

        let types = relation_node::find_many(&self.neo4j)
            .space_id(prop_filter::value(self.space_id.clone()))
            .from_id(prop_filter::value(self.entity_id.clone()))
            .relation_type(prop_filter::value(system_ids::TYPES_ATTRIBUTE))
            .send()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        Ok(attributes.map(|data| {
            Entity::new(self.entity_id, data).with_types(types.into_iter().map(|r| r.to))
        }))
    }
}

pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    filter: EntityFilter,
    limit: usize,
    skip: Option<usize>,
    order_by: Option<FieldOrderBy>,

    space_id: PropFilter<String>,
    version: VersionFilter,
}

impl FindManyQuery {
    fn new(neo4j: neo4rs::Graph, space_id: String, version: Option<String>) -> Self {
        FindManyQuery {
            neo4j,
            filter: EntityFilter::default(),
            limit: 100,
            skip: None,
            order_by: None,
            space_id: prop_filter::value(space_id),
            version: VersionFilter::new(version),
        }
    }

    pub fn id(mut self, id: PropFilter<String>) -> Self {
        self.filter.id = Some(id);
        self
    }

    pub fn attribute(mut self, attribute: AttributeFilter) -> Self {
        self.filter.attributes.push(attribute);
        self
    }

    pub fn attribute_mut(&mut self, attribute: AttributeFilter) {
        self.filter.attributes.push(attribute);
    }

    pub fn attributes(mut self, attributes: impl IntoIterator<Item = AttributeFilter>) -> Self {
        self.filter.attributes.extend(attributes);
        self
    }

    pub fn attributes_mut(&mut self, attributes: impl IntoIterator<Item = AttributeFilter>) {
        self.filter.attributes.extend(attributes);
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    pub fn order_by(mut self, order_by: FieldOrderBy) -> Self {
        self.order_by = Some(order_by);
        self
    }

    pub fn order_by_mut(&mut self, order_by: FieldOrderBy) {
        self.order_by = Some(order_by);
    }

    /// Overwrite the current filter with a new one
    pub fn with_filter(mut self, filter: EntityFilter) -> Self {
        self.filter = filter;
        self
    }

    fn into_query_part(self) -> QueryPart {
        let mut query_part = QueryPart::default()
            .match_clause("(e:Entity)")
            .limit(self.limit);

        query_part.merge_mut(
            self.filter
                .space_id(self.space_id.clone())
                .into_query_part("e"),
        );

        if let Some(order_by) = self.order_by {
            query_part.merge_mut(order_by.into_query_part("e"));
        }

        if let Some(skip) = self.skip {
            query_part = query_part.skip(skip);
        }

        query_part.with_clause("DISTINCT e", {
            QueryPart::default()
                .match_clause("(e) -[r:ATTRIBUTE]-> (n:Attribute)")
                .merge(self.space_id.into_query_part("r", "space_id"))
                .merge(self.version.into_query_part("r"))
                .with_clause(
                    "e, collect(n{.*}) AS attrs",
                    query_part::return_query("e{.*, attributes: attrs}"),
                )
        })
    }
}

impl<T: FromAttributes> QueryStream<Entity<T>> for FindManyQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Entity<T>, DatabaseError>>, DatabaseError> {
        let neo4j = self.neo4j.clone();

        let query = if cfg!(debug_assertions) || cfg!(test) {
            let query_part = self.into_query_part();
            tracing::info!("entity::FindManyQuery:\n{}", query_part);
            query_part.build()
        } else {
            self.into_query_part().build()
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
    async fn test_insert_find_one() {
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
                Triple::new("foo_type", "name", "Foo"),
                Triple::new(system_ids::TYPES_ATTRIBUTE, "name", "Types"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let entity = Entity::new("abc", foo).with_type("foo_type");

        entity
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert entity");

        let found_entity = find_one(&neo4j, "abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find entity")
            .expect("Entity not found");

        assert_eq!(found_entity, entity);
    }

    #[tokio::test]
    async fn test_insert_find_many() {
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
                Triple::new("foo_type", "name", "Foo"),
                Triple::new(system_ids::TYPES_ATTRIBUTE, "name", "Types"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let entity = Entity::new("abc", foo).with_type("foo_type");

        entity
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert entity");

        let stream = find_many(&neo4j, "ROOT", None)
            .attribute(AttributeFilter::new("name").value(prop_filter::value("Alice")))
            .limit(1)
            .send()
            .await
            .expect("Failed to find entity");

        pin_mut!(stream);

        let found_entity: Entity<Foo> = stream
            .next()
            .await
            .expect("Failed to get next entity")
            .expect("Entity not found");

        assert_eq!(found_entity.node.id, entity.node.id);
        assert_eq!(found_entity.attributes, entity.attributes);
    }
}
