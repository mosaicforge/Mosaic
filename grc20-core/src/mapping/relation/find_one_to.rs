use crate::{
    entity::utils::MatchEntity, error::DatabaseError, mapping::{
        prop_filter,
        query_utils::{query_part, QueryPart, VersionFilter},
        AttributeNode, Entity, EntityNode, FromAttributes, Query,
    }, relation::utils::MatchOneRelation
};

pub struct FindOneToQuery<T> {
    pub(super) neo4j: neo4rs::Graph,
    pub(super) id: String,
    pub(super) space_id: String,
    pub(super) space_version: VersionFilter,
    pub(super) _phantom: std::marker::PhantomData<T>,
}

impl<T> FindOneToQuery<T> {
    pub(super) fn new(
        neo4j: &neo4rs::Graph,
        id: String,
        space_id: String,
        space_version: Option<String>,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id,
            space_id,
            space_version: VersionFilter::new(space_version),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Query<Option<EntityNode>> for FindOneToQuery<EntityNode> {
    async fn send(self) -> Result<Option<EntityNode>, DatabaseError> {
        let neo4j = self.neo4j.clone();

        let query = QueryPart::default()
            .match_clause(
                "(from:Entity) -[r:RELATION {id: $id, space_id: $space_id}]-> (to:Entity)",
            )
            .merge(self.space_version.compile("r"))
            .return_clause("to")
            .limit(1)
            .order_by_clause("r.index")
            .params("id", self.id)
            .params("space_id", self.space_id)
            .build();

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            to: EntityNode,
        }

        neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                Result::<_, DatabaseError>::Ok(row.to)
            })
            .transpose()
    }
}

impl<T: FromAttributes> Query<Option<Entity<T>>> for FindOneToQuery<Entity<T>> {
    async fn send(self) -> Result<Option<Entity<T>>, DatabaseError> {
        let match_relation =
            MatchOneRelation::new(self.id.clone(), self.space_id.clone(), &self.space_version);

        let space_filter = Some(prop_filter::value(self.space_id.clone()));
        let match_entity = MatchEntity::new(&space_filter, &self.space_version);

        let query = match_relation.chain(
            "from",
            "to",
            "r",
            match_entity.chain(
                "to",
                "attrs",
                "types",
                query_part::return_query("to{.*, attrs: attrs, types: types}"),
            ),
        );

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
            relation::{find_one_to, RelationEdge},
            triple, Entity, EntityNode, Query, Triple,
        },
        system_ids,
    };

    #[tokio::test]
    async fn test_find_one_to_node() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        triple::insert_many(&neo4j, &BlockMetadata::default(), "space_id", "0")
            .triples(vec![
                Triple::new("alice", "name", "Alice"),
                Triple::new("bob", "name", "Bob"),
                Triple::new("knows", "name", "knows"),
                Triple::new("charlie", "name", "Charlie"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation_edge = RelationEdge::new("abc", "alice", "bob", "knows", "0");

        relation_edge
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let found_relation = find_one_to::<EntityNode>(&neo4j, "abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find relation")
            .expect("Relation not found");

        assert_eq!(
            found_relation,
            EntityNode {
                id: "bob".to_string(),
                system_properties: BlockMetadata::default().into(),
            },
        );
    }

    #[tokio::test]
    async fn test_find_one_to_entity() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        #[derive(Clone, Debug, PartialEq)]
        struct Person {
            name: String,
        }

        impl mapping::IntoAttributes for Person {
            fn into_attributes(
                self,
            ) -> Result<mapping::Attributes, mapping::TriplesConversionError> {
                Ok(mapping::Attributes::default().attribute(("name", self.name)))
            }
        }

        impl mapping::FromAttributes for Person {
            fn from_attributes(
                mut attributes: mapping::Attributes,
            ) -> Result<Self, mapping::TriplesConversionError> {
                Ok(Self {
                    name: attributes.pop("name")?,
                })
            }
        }

        triple::insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .triples(vec![
                Triple::new("person_type", "name", "Person"),
                Triple::new("knows", "name", "knows"),
                Triple::new(system_ids::TYPES_ATTRIBUTE, "name", "Types"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        Entity::new(
            "alice",
            Person {
                name: "Alice".to_string(),
            },
        )
        .with_type("person_type")
        .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
        .send()
        .await
        .expect("Failed to insert entity");

        Entity::new(
            "bob",
            Person {
                name: "Bob".to_string(),
            },
        )
        .with_type("person_type")
        .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
        .send()
        .await
        .expect("Failed to insert entity");

        let relation_edge = RelationEdge::new("abc", "alice", "bob", "knows", "0");

        relation_edge
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let found_relation = find_one_to::<Entity<Person>>(&neo4j, "abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find relation")
            .expect("Relation not found");

        assert_eq!(
            found_relation,
            Entity {
                node: EntityNode {
                    id: "bob".to_string(),
                    system_properties: BlockMetadata::default().into(),
                },
                attributes: Person {
                    name: "Bob".to_string()
                },
                types: vec!["person_type".to_string()],
            },
        );
    }
}
