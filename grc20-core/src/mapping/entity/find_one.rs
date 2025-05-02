use crate::{
    entity::utils::MatchEntity,
    error::DatabaseError,
    mapping::{
        prop_filter,
        query_utils::{query_builder::{MatchQuery, QueryBuilder, Subquery}, VersionFilter},
        AttributeNode, FromAttributes, Query,
    },
};

use super::{Entity, EntityNode};

pub struct FindOneQuery<T> {
    neo4j: neo4rs::Graph,
    id: String,
    space_id: Option<String>,
    version: VersionFilter,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> FindOneQuery<T> {
    pub(super) fn new(neo4j: &neo4rs::Graph, id: String) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id,
            space_id: None,
            version: VersionFilter::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn space_id(mut self, space_id: impl Into<String>) -> Self {
        self.space_id = Some(space_id.into());
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version.version_mut(version.into());
        self
    }

    pub fn version_opt(mut self, version: Option<String>) -> Self {
        self.version.version_opt(version);
        self
    }
}

impl Query<Option<EntityNode>> for FindOneQuery<EntityNode> {
    async fn send(self) -> Result<Option<EntityNode>, DatabaseError> {
        const QUERY: &str = r#"
            MATCH (e:Entity {id: $id})
            RETURN e
        "#;

        let query = neo4rs::query(QUERY).param("id", self.id);

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            e: EntityNode,
        }

        self.neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                Result::<_, DatabaseError>::Ok(row.e)
            })
            .transpose()
    }
}

impl<T: FromAttributes> Query<Option<Entity<T>>> for FindOneQuery<Entity<T>> {
    async fn send(self) -> Result<Option<Entity<T>>, DatabaseError> {
        let space_filter = self.space_id.map(prop_filter::value);
        let match_entity = MatchEntity::new(&space_filter, &self.version);

        let query = QueryBuilder::default()
            .subquery(MatchQuery::new("(e:Entity {id: $id})"))
            .with(
                vec!["e".to_string()],
                match_entity.chain(
                    "e",
                    "attrs",
                    "types",
                    "RETURN e{.*, attrs: attrs, types: types}",
                ),
            )
            .params("id", self.id);

        if cfg!(debug_assertions) || cfg!(test) {
            println!(
                "entity::FindOneQuery::<Entity<T>>:\n{}\nparams:{:?}",
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

        self.neo4j
            .execute(query.build())
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                Result::<_, DatabaseError>::Ok(Entity {
                    node: row.node,
                    attributes: T::from_attributes(row.attrs.into())?,
                    types: row.types.into_iter().map(|t| t.id).collect(),
                })
            })
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        block::BlockMetadata,
        mapping::{
            self,
            entity::{find_one, models::SystemProperties},
            triple, Entity, EntityNode, Query, Triple,
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
    async fn test_find_by_id() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        let triple = Triple {
            entity: "abc".to_string(),
            attribute: "name".to_string(),
            value: "Alice".into(),
        };

        triple
            .insert(&neo4j, &BlockMetadata::default(), "space_id", "0")
            .send()
            .await
            .expect("Failed to insert triple");

        let entity = find_one::<EntityNode>(&neo4j, "abc")
            .send()
            .await
            .expect("Failed to find entity")
            .expect("Entity not found");

        assert_eq!(
            entity,
            EntityNode {
                id: "abc".to_string(),
                system_properties: SystemProperties::default(),
            }
        );
    }

    #[tokio::test]
    async fn test_insert_find_one() {
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
        println!("Entity: {:?}", entity);

        entity
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert entity");

        let found_entity = find_one::<Entity<Foo>>(&neo4j, "abc")
            .space_id("ROOT")
            .send()
            .await
            .expect("Failed to find entity")
            .expect("Entity not found");

        assert_eq!(found_entity, entity);
    }
}
