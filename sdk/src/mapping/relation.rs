use std::collections::HashMap;

use futures::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::{
    error::DatabaseError, indexer_ids, mapping::query_utils::query_part::IntoQueryPart,
    models::BlockMetadata, neo4j_utils::serde_value_to_bolt, system_ids,
};

use super::{attributes::SystemProperties, relation_queries, Entity, Triples};

pub struct Relation<T> {
    // pub id: String,
    pub r#type: String,
    pub from: String,
    pub to: String,
    // pub attributes: Attributes<T>,
    pub entity: Entity<T>,
}

impl<T> Relation<T> {
    pub fn new(
        id: &str,
        space_id: &str,
        r#type: &str,
        from: &str,
        to: &str,
        block: &BlockMetadata,
        data: T,
    ) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            r#type: r#type.to_string(),
            entity: Entity::new(id, space_id, block, data).with_type(system_ids::RELATION_TYPE),
        }
    }

    pub fn from_entity(entity: Entity<T>, from: &str, to: &str, r#type: &str) -> Self {
        Self {
            // id: entity.id().to_string(),
            from: from.to_string(),
            to: to.to_string(),
            r#type: r#type.to_string(),
            // attributes: entity.attributes,
            entity,
        }
    }

    pub fn id(&self) -> &str {
        self.entity.id()
    }

    pub fn space_id(&self) -> &str {
        self.entity.space_id()
    }

    pub fn attributes(&self) -> &T {
        self.entity.attributes()
    }

    pub fn attributes_mut(&mut self) -> &mut T {
        self.entity.attributes_mut()
    }

    pub fn system_properties(&self) -> &SystemProperties {
        &self.entity.attributes.system_properties
    }

    pub fn with_type(mut self, type_id: &str) -> Self {
        self.entity = self.entity.with_type(type_id);
        self
    }

    /// Returns a query to delete the current relation
    pub fn delete_query(id: &str) -> neo4rs::Query {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (from) <-[:`{FROM_ENTITY}`]- (r {{id: $id}}) -[:`{TO_ENTITY}`]-> (to)
            REMOVE from:$(to.id)
            DETACH DELETE r
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
        );

        neo4rs::query(QUERY).param("id", id)
    }

    pub async fn types(
        &self,
        neo4j: &neo4rs::Graph,
    ) -> Result<Vec<Entity<Triples>>, DatabaseError> {
        Self::find_types(neo4j, self.id(), self.space_id()).await
    }

    pub async fn find_types(
        neo4j: &neo4rs::Graph,
        id: &str,
        space_id: &str,
    ) -> Result<Vec<Entity<Triples>>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (r {{id: $id, space_id: $space_id}})
            UNWIND labels(r) as l
            MATCH (t {{id: l, space_id: $space_id}})
            RETURN t
            "#,
        );

        let query = neo4rs::query(QUERY)
            .param("id", id)
            .param("space_id", space_id);

        #[derive(Debug, Deserialize)]
        struct RowResult {
            t: neo4rs::Node,
        }

        neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move { Ok(row.t.try_into()?) })
            .try_collect::<Vec<_>>()
            .await
    }

    pub async fn to<E>(&self, neo4j: &neo4rs::Graph) -> Result<Option<Entity<E>>, DatabaseError>
    where
        E: for<'a> Deserialize<'a> + Send,
    {
        Self::find_to(neo4j, self.id(), self.space_id()).await
    }

    pub async fn find_to<E>(
        neo4j: &neo4rs::Graph,
        id: &str,
        space_id: &str,
    ) -> Result<Option<Entity<E>>, DatabaseError>
    where
        E: for<'a> Deserialize<'a> + Send,
    {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH ({{id: $id, space_id: $space_id}}) -[:`{TO_ENTITY}`]-> (to)
            RETURN to
            "#,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
        );

        let query = neo4rs::query(QUERY)
            .param("id", id)
            .param("space_id", space_id);

        #[derive(Debug, Deserialize)]
        struct RowResult {
            to: neo4rs::Node,
        }

        Ok(neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                row.to.try_into()
            })
            .transpose()?)
    }

    pub async fn from<E>(&self, neo4j: &neo4rs::Graph) -> Result<Option<Entity<E>>, DatabaseError>
    where
        E: for<'a> Deserialize<'a> + Send,
    {
        Self::find_from(neo4j, self.id(), self.space_id()).await
    }

    pub async fn find_from<E>(
        neo4j: &neo4rs::Graph,
        id: &str,
        space_id: &str,
    ) -> Result<Option<Entity<E>>, DatabaseError>
    where
        E: for<'a> Deserialize<'a> + Send,
    {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH ({{id: $id, space_id: $space_id}}) -[:`{FROM_ENTITY}`]-> (from)
            RETURN from
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
        );

        let query = neo4rs::query(QUERY)
            .param("id", id)
            .param("space_id", space_id);

        #[derive(Debug, Deserialize)]
        struct RowResult {
            from: neo4rs::Node,
        }

        Ok(neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                row.from.try_into()
            })
            .transpose()?)
    }
}

impl<T> Relation<T>
where
    T: Serialize,
{
    /// Upsert the current relation
    pub async fn upsert(&self, neo4j: &neo4rs::Graph) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MERGE (from {{id: $from_id, space_id: $space_id}})
            ON CREATE SET from += {{
                `{CREATED_AT}`: datetime($created_at),
                `{CREATED_AT_BLOCK}`: $created_at_block
            }}
            SET from:$($from_types)
            SET from += {{
                `{UPDATED_AT}`: datetime($updated_at),
                `{UPDATED_AT_BLOCK}`: $updated_at_block
            }}
            MERGE (to {{id: $to_id, space_id: $space_id}})
            ON CREATE SET to += {{
                `{CREATED_AT}`: datetime($created_at),
                `{CREATED_AT_BLOCK}`: $created_at_block
            }}
            SET to += {{
                `{UPDATED_AT}`: datetime($updated_at),
                `{UPDATED_AT_BLOCK}`: $updated_at_block
            }}
            MERGE (relation_type {{id: $relation_type_id, space_id: $space_id}})
            ON CREATE SET relation_type += {{
                `{CREATED_AT}`: datetime($created_at),
                `{CREATED_AT_BLOCK}`: $created_at_block
            }}
            SET relation_type += {{
                `{UPDATED_AT}`: datetime($updated_at),
                `{UPDATED_AT_BLOCK}`: $updated_at_block
            }}
            MERGE (from)<-[:`{FROM_ENTITY}`]-(r {{id: $id, space_id: $space_id}})-[:`{TO_ENTITY}`]->(to)
            MERGE (r) -[:`{RELATION_TYPE}`]-> (relation_type)
            ON CREATE SET r += {{
                `{CREATED_AT}`: datetime($created_at),
                `{CREATED_AT_BLOCK}`: $created_at_block
            }}
            SET r:$($r_types)
            SET r += {{
                `{UPDATED_AT}`: datetime($updated_at),
                `{UPDATED_AT_BLOCK}`: $updated_at_block
            }}
            SET r += $data
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let bolt_data = match serde_value_to_bolt(serde_json::to_value(self.attributes())?) {
            neo4rs::BoltType::Map(map) => neo4rs::BoltType::Map(map),
            _ => neo4rs::BoltType::Map(Default::default()),
        };

        let query = neo4rs::query(QUERY)
            .param("id", self.id())
            .param("space_id", self.space_id())
            .param("from_id", self.from.clone())
            .param("to_id", self.to.clone())
            .param("space_id", self.space_id())
            .param(
                "created_at",
                self.system_properties().created_at.to_rfc3339(),
            )
            .param(
                "created_at_block",
                self.system_properties().created_at_block.to_string(),
            )
            .param(
                "updated_at",
                self.system_properties().updated_at.to_rfc3339(),
            )
            .param(
                "updated_at_block",
                self.system_properties().updated_at_block.to_string(),
            )
            .param("relation_type_id", self.r#type.clone())
            .param("r_types", self.entity.types.clone())
            .param(
                "from_types",
                if self.r#type == system_ids::TYPES_ATTRIBUTE {
                    vec![self.to.clone()]
                } else {
                    vec![]
                },
            )
            .param("data", bolt_data);

        Ok(neo4j.run(query).await?)
    }
}

impl<T> Relation<T>
where
    T: for<'a> Deserialize<'a>,
{
    /// Returns the entity with the given ID, if it exists
    pub async fn find_by_id(
        neo4j: &neo4rs::Graph,
        id: &str,
        space_id: &str,
    ) -> Result<Option<Self>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (from) <-[:`{FROM_ENTITY}`]- (r:`{RELATION}` {{ id: $id, space_id: $space_id }}) -[:`{TO_ENTITY}`]-> (to)
            MATCH (r) -[:`{RELATION_TYPE}`]-> (rt)
            RETURN from, r, to, rt
            "#,
            RELATION = system_ids::RELATION_TYPE,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
        );

        let query = neo4rs::query(QUERY)
            .param("id", id)
            .param("space_id", space_id);

        Self::_find_one(neo4j, query).await
    }

    /// Returns the entities from the given list of IDs
    pub async fn find_by_ids(
        neo4j: &neo4rs::Graph,
        ids: &[String],
    ) -> Result<Vec<Self>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            UNWIND $ids AS id
            MATCH (from) <-[:`{FROM_ENTITY}`]- (r:`{RELATION}` {{ id: $id, space_id: $space_id }}) -[:`{TO_ENTITY}`]-> (to)
            MATCH (r) -[:`{RELATION_TYPE}`]-> (rt)
            RETURN from, r, to, rt
            "#,
            RELATION = system_ids::RELATION_TYPE,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
        );

        let query = neo4rs::query(QUERY).param("ids", ids);

        Self::_find_many(neo4j, query).await
    }

    /// Returns the entities with the given relation types
    pub async fn find_by_relation_types(
        neo4j: &neo4rs::Graph,
        relation_types: &[String],
        space_id: &str,
    ) -> Result<Vec<Self>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            UNWIND $relation_types AS relation_type
            MATCH (from {{space_id: $space_id}}) <-[:`{FROM_ENTITY}`]- (r:`{RELATION}` {{space_id: $space_id}}) -[:`{TO_ENTITY}`]-> (to {{space_id: $space_id}})
            MATCH (r) -[:`{RELATION_TYPE}`]-> (rt {{id: $relation_type, space_id: $space_id}})
            RETURN from, r, to, rt
            "#,
            RELATION = system_ids::RELATION_TYPE,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
        );

        let query = neo4rs::query(QUERY)
            .param("relation_types", relation_types)
            .param("space_id", space_id);

        Self::_find_many(neo4j, query).await
    }

    pub async fn find_many(
        neo4j: &neo4rs::Graph,
        r#where: Option<relation_queries::FindMany>,
    ) -> Result<Vec<Self>, DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (from) <-[:`{FROM_ENTITY}`]- (r:`{RELATION}`) -[:`{TO_ENTITY}`]-> (to)
            MATCH (r) -[:`{RELATION_TYPE}`]-> (rt)
            RETURN from, r, to, rt
            LIMIT 100
            "#,
            RELATION = system_ids::RELATION_TYPE,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
        );

        if let Some(filter) = r#where {
            Self::_find_many(neo4j, filter.into_query_part().build()).await
        } else {
            Self::_find_many(neo4j, neo4rs::query(QUERY)).await
        }
    }

    async fn _find_one(
        neo4j: &neo4rs::Graph,
        query: neo4rs::Query,
    ) -> Result<Option<Self>, DatabaseError> {
        #[derive(Debug, Deserialize)]
        struct RowResult {
            from: neo4rs::Node,
            r: neo4rs::Node,
            to: neo4rs::Node,
            rt: neo4rs::Node,
        }

        neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;

                let from: Entity<()> = row.from.try_into()?;
                let rel: Entity<T> = row.r.try_into()?;
                let to: Entity<()> = row.to.try_into()?;
                let rt: Entity<()> = row.rt.try_into()?;

                Ok(Relation::from_entity(rel, from.id(), to.id(), rt.id()))
            })
            .transpose()
    }

    async fn _find_many(
        neo4j: &neo4rs::Graph,
        query: neo4rs::Query,
    ) -> Result<Vec<Self>, DatabaseError> {
        #[derive(Debug, Deserialize)]
        struct RowResult {
            from: neo4rs::Node,
            r: neo4rs::Node,
            to: neo4rs::Node,
            rt: neo4rs::Node,
        }

        neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move {
                let from: Entity<()> = row.from.try_into()?;
                let rel: Entity<T> = row.r.try_into()?;
                let to: Entity<()> = row.to.try_into()?;
                let rt: Entity<()> = row.rt.try_into()?;

                Ok(Relation::from_entity(rel, from.id(), to.id(), rt.id()))
            })
            .try_collect::<Vec<_>>()
            .await
    }
}

impl Relation<HashMap<String, neo4rs::BoltType>> {
    pub fn with_attribute<T>(mut self, key: String, value: T) -> Self
    where
        T: Into<neo4rs::BoltType>,
    {
        self.attributes_mut().insert(key, value.into());
        self
    }
}
