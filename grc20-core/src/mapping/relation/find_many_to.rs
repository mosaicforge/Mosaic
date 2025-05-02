use futures::{Stream, StreamExt, TryStreamExt};

use crate::{
    entity::utils::MatchEntity,
    error::DatabaseError,
    mapping::{
        query_utils::{query_part, QueryPart, VersionFilter},
        AttributeNode, Entity, EntityNode, FromAttributes, PropFilter, QueryStream,
    },
};

use super::utils::RelationFilter;

pub struct FindManyToQuery<T> {
    pub(super) neo4j: neo4rs::Graph,
    pub(super) filter: RelationFilter,

    pub(super) space_id: Option<PropFilter<String>>,
    pub(super) version: VersionFilter,

    pub(super) limit: usize,
    pub(super) skip: Option<usize>,

    pub(super) _phantom: std::marker::PhantomData<T>,
}

impl<T> FindManyToQuery<T> {
    pub(super) fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            filter: RelationFilter::default(),
            space_id: None,
            version: VersionFilter::default(),
            limit: 100,
            skip: None,
            _phantom: std::marker::PhantomData,
        }
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

    fn compile(&self) -> QueryPart {
        QueryPart::default()
            .match_clause("(from:Entity) -[r:RELATION]-> (to:Entity)")
            .merge(self.filter.compile("r", "from", "to"))
            .merge(self.version.compile("r"))
            .merge_opt(
                self.space_id
                    .as_ref()
                    .map(|space_id| space_id.compile("r", "space_id", None)),
            )
            .order_by_clause("r.index")
            .limit(self.limit)
            .skip_opt(self.skip)
    }
}

impl QueryStream<EntityNode> for FindManyToQuery<EntityNode> {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<EntityNode, DatabaseError>>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query_part = self.compile().return_clause("to");

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!("relation_node::FindManyToQuery:\n{}", query_part);
        };
        let query = query_part.build();

        Ok(neo4j
            .execute(query)
            .await?
            .into_stream_as::<EntityNode>()
            .map_err(DatabaseError::from))
    }
}

impl<T: FromAttributes> QueryStream<Entity<T>> for FindManyToQuery<Entity<T>> {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Entity<T>, DatabaseError>>, DatabaseError> {
        let neo4j = self.neo4j.clone();

        let match_entity = MatchEntity::new(&self.space_id, &self.version);

        let query_part = self.compile().with_clause(
            "to",
            match_entity.chain(
                "to",
                "attrs",
                "types",
                query_part::return_query("to{.*, attrs: attrs, types: types}"),
            ),
        );

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!("relation_node::FindManyToQuery:\n{}", query_part);
        };
        let query = query_part.build();

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            node: EntityNode,
            attrs: Vec<AttributeNode>,
            types: Vec<EntityNode>,
        }

        let stream = neo4j
            .execute(query)
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
    use futures::TryStreamExt;

    use crate::{
        block::BlockMetadata,
        mapping::{
            self, prop_filter,
            relation::{find_many, insert_many, RelationEdge},
            triple, Entity, EntityFilter, EntityNode, EntityNodeRef, Query, QueryStream, Triple,
        },
        relation::utils::RelationFilter,
        system_ids,
    };

    #[tokio::test]
    async fn test_find_many_to_entity() {
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

        Entity::<Person>::new(
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

        Entity::<Person>::new(
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

        Entity::<Person>::new(
            "charlie",
            Person {
                name: "Charlie".to_string(),
            },
        )
        .with_type("person_type")
        .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
        .send()
        .await
        .expect("Failed to insert entity");

        let relation_edges = vec![
            RelationEdge::new("abc", "alice", "bob", "knows", "0"),
            RelationEdge::new("dev", "alice", "charlie", "knows", "1"),
        ];

        insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .relations(relation_edges.clone())
            .send()
            .await
            .expect("Failed to insert relations");

        let found_relations = find_many::<EntityNodeRef>(&neo4j)
            .filter(
                RelationFilter::default()
                    .relation_type(EntityFilter::default().id(prop_filter::value("knows")))
                    .from_(EntityFilter::default().id(prop_filter::value("alice"))),
            )
            .select_to::<Entity<Person>>()
            .send()
            .await
            .expect("Failed to find relations")
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to collect triples");

        assert_eq!(
            found_relations,
            vec![
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
                Entity {
                    node: EntityNode {
                        id: "charlie".to_string(),
                        system_properties: BlockMetadata::default().into(),
                    },
                    attributes: Person {
                        name: "Charlie".to_string()
                    },
                    types: vec!["person_type".to_string()],
                }
            ],
        );
    }
}
