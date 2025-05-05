use futures::{Stream, StreamExt, TryStreamExt};

use crate::{
    entity::utils::MatchEntity,
    error::DatabaseError,
    mapping::{
        order_by::FieldOrderBy,
        query_utils::{
            query_builder::{MatchQuery, QueryBuilder, Subquery},
            VersionFilter,
        },
        AttributeFilter, AttributeNode, EntityFilter, FromAttributes, PropFilter, QueryStream,
    },
};

use super::{Entity, EntityNode};

pub struct FindManyQuery<T> {
    neo4j: neo4rs::Graph,
    filter: EntityFilter,
    order_by: Option<FieldOrderBy>,
    limit: usize,
    skip: Option<usize>,

    space_id: Option<PropFilter<String>>,
    version: VersionFilter,

    _marker: std::marker::PhantomData<T>,
}

impl<T> FindManyQuery<T> {
    pub(super) fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            filter: EntityFilter::default(),
            order_by: None,
            limit: 100,
            skip: None,
            space_id: None,
            version: VersionFilter::default(),
            _marker: std::marker::PhantomData,
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

    /// Overwrite the current filter with a new one
    pub fn with_filter(mut self, filter: EntityFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn order_by(mut self, order_by: FieldOrderBy) -> Self {
        self.order_by = Some(order_by);
        self
    }

    pub fn order_by_mut(&mut self, order_by: FieldOrderBy) {
        self.order_by = Some(order_by);
    }

    pub fn space_id(mut self, space_id: impl Into<PropFilter<String>>) -> Self {
        self.space_id = Some(space_id.into());
        self
    }

    pub fn version(mut self, space_version: String) -> Self {
        self.version.version_mut(space_version);
        self
    }

    pub fn version_opt(mut self, space_version: Option<String>) -> Self {
        self.version.version_opt(space_version);
        self
    }

    fn subquery(&self) -> QueryBuilder {
        QueryBuilder::default()
            .subquery(MatchQuery::new("(e:Entity)"))
            .subquery(self.filter.subquery("e"))
            .subquery_opt(self.order_by.as_ref().map(|o| o.subquery("e")))
            .limit(self.limit)
            .skip_opt(self.skip)
    }
}

impl QueryStream<EntityNode> for FindManyQuery<EntityNode> {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<EntityNode, DatabaseError>>, DatabaseError> {
        let neo4j = self.neo4j.clone();

        let query = self.subquery().r#return("DISTINCT e");

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!(
                "entity_node::FindManyQuery::<EntityNode>:\n{}\nparams:{:?}",
                query.compile(),
                query.params()
            );
        };

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            e: EntityNode,
        }

        Ok(neo4j
            .execute(query.build())
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move { Ok(row.e) }))
    }
}

impl<T: FromAttributes> QueryStream<Entity<T>> for FindManyQuery<Entity<T>> {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Entity<T>, DatabaseError>>, DatabaseError> {
        let match_entity = MatchEntity::new(&self.space_id, &self.version);

        let query = self.subquery().with(
            vec!["e".to_string()],
            match_entity.chain(
                "e",
                "attrs",
                "types",
                "RETURN e{.*, attrs: attrs, types: types}",
            ),
        );

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!(
                "entity_node::FindManyQuery::<Entity<T>>:\n{}\nparams:{:?}",
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
                        .map(|data| Entity {
                            node: row.node,
                            attributes: data,
                            types: row.types.into_iter().map(|t| t.id).collect(),
                        })
                        .map_err(DatabaseError::from)
                })
            });

        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use futures::{pin_mut, StreamExt};

    use crate::{
        block::BlockMetadata,
        mapping::{
            self, entity::find_many, prop_filter, triple, AttributeFilter, Entity, Query,
            QueryStream, Triple,
        },
        system_ids,
    };

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
    async fn test_insert_find_many() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

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

        let stream = find_many::<Entity<Foo>>(&neo4j)
            .space_id("ROOT")
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
