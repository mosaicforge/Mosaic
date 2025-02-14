use futures::{stream, StreamExt, TryStreamExt};

use crate::{error::DatabaseError, ids, mapping::EntityNode, models::BlockMetadata, system_ids};

use super::{
    attributes::{self, FromAttributes, IntoAttributes},
    query_utils::{AttributeFilter, PropFilter, Query, QueryPart},
    relation_node, RelationNode,
};

/// High level model encapsulating an entity and its attributes.
pub struct Entity<T> {
    pub id: String,
    pub data: T,
    pub types: Vec<String>,
}

impl<T> Entity<T> {
    pub fn new(id: impl Into<String>, data: T) -> Self {
        Entity {
            id: id.into(),
            data,
            types: vec![],
        }
    }

    pub fn with_type(mut self, r#type: impl Into<String>) -> Self {
        self.types.push(r#type.into());
        self
    }

    pub fn insert(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: i64,
    ) -> InsertOneQuery<T> {
        InsertOneQuery::new(neo4j.clone(), block.clone(), self, space_id, space_version)
    }
}

pub fn find_one(
    neo4j: &neo4rs::Graph,
    space_id: impl Into<String>,
    entity_id: impl Into<String>,
    space_version: Option<i64>,
) -> FindOneQuery {
    FindOneQuery::new(
        neo4j.clone(),
        space_id.into(),
        entity_id.into(),
        space_version,
    )
}

pub fn find_many(
    neo4j: &neo4rs::Graph,
    space_id: impl Into<String>,
    space_version: Option<i64>,
) -> FindManyQuery {
    FindManyQuery::new(neo4j.clone(), space_id, space_version)
}

pub struct InsertOneQuery<T> {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    entity: Entity<T>,
    space_id: String,
    space_version: i64,
}

impl<T> InsertOneQuery<T> {
    pub fn new(
        neo4j: neo4rs::Graph,
        block: BlockMetadata,
        entity: Entity<T>,
        space_id: String,
        space_version: i64,
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
            &self.entity.id,
            &self.space_id,
            self.space_version,
            self.entity.data,
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
                    &ids::create_id_from_unique_string(&format!(
                        "{}:{}:{}:{}",
                        self.space_id,
                        self.entity.id,
                        system_ids::TYPES_ATTRIBUTE,
                        t,
                    )),
                    &self.entity.id,
                    t,
                    system_ids::TYPES_ATTRIBUTE,
                    "0".into(),
                )
            })
            .collect::<Vec<_>>();

        // Insert the relations
        relation_node::insert_many(
            &self.neo4j,
            &self.block,
            &self.space_id,
            self.space_version,
            types_relations,
        )
        .send()
        .await?;

        Ok(())
    }
}

pub struct FindOneQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
    entity_id: String,
    space_version: Option<i64>,
}

impl FindOneQuery {
    pub fn new(
        neo4j: neo4rs::Graph,
        space_id: String,
        entity_id: String,
        space_version: Option<i64>,
    ) -> Self {
        FindOneQuery {
            neo4j,
            space_id,
            entity_id,
            space_version,
        }
    }
}

impl<T: FromAttributes> Query<Option<Entity<T>>> for FindOneQuery {
    async fn send(self) -> Result<Option<Entity<T>>, DatabaseError> {
        let attributes = attributes::find_one(
            &self.neo4j,
            self.space_id,
            self.entity_id.clone(),
            self.space_version,
        )
        .send()
        .await?;

        Ok(attributes.map(|data| Entity::new(self.entity_id, data)))
    }
}

pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
    space_version: Option<i64>,
    id: Option<PropFilter<String>>,
    attributes: Vec<AttributeFilter>,
}

impl FindManyQuery {
    pub fn new(
        neo4j: neo4rs::Graph,
        space_id: impl Into<String>,
        space_version: Option<i64>,
    ) -> Self {
        FindManyQuery {
            neo4j,
            space_id: space_id.into(),
            space_version,
            id: None,
            attributes: vec![],
        }
    }

    pub fn id(mut self, id: PropFilter<String>) -> Self {
        self.id = Some(id);
        self
    }

    pub fn attribute(mut self, attribute: AttributeFilter) -> Self {
        self.attributes.push(attribute);
        self
    }

    fn into_query_part(self) -> QueryPart {
        let mut query_part = QueryPart::default().match_clause("(e)").return_clause("e");

        if let Some(id) = self.id {
            query_part.merge_mut(id.into_query_part("e", "id"));
        }

        for attribute in self.attributes {
            query_part.merge_mut(attribute.into_query_part("e"));
        }

        query_part
    }
}

// TODO: (optimization) Turn this into a stream instead of returning vec
impl<T: FromAttributes> Query<Vec<T>> for FindManyQuery {
    async fn send(self) -> Result<Vec<T>, DatabaseError> {
        let neo4j = &self.neo4j.clone();
        let space_id = &self.space_id.clone();
        let space_version = self.space_version.clone();

        let query = self.into_query_part().build();

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            e: EntityNode,
        }

        let entity_nodes = neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move { Ok(row.e) })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(stream::iter(entity_nodes)
            .map(|entity| async move {
                entity
                    .get_attributes(neo4j, space_id, space_version)
                    .send()
                    .await
                    .map_err(DatabaseError::from)
            })
            .buffered(18)
            .try_collect::<Vec<Option<T>>>()
            .await?
            .into_iter()
            .filter_map(|attributes| attributes)
            .collect())
    }
}
