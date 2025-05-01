use std::collections::HashMap;

use futures::{Stream, StreamExt, TryStreamExt};
use neo4rs::BoltType;
use serde::Deserialize;

use crate::{block::BlockMetadata, error::DatabaseError, indexer_ids, mapping::{prop_filter, relation_queries::MatchOneRelation}, pb, system_ids};

use super::{
    attributes, entity_node::{self, EntityNodeRef, SystemProperties}, entity_queries::MatchEntity, query_utils::{query_part, PropFilter, Query, QueryPart, QueryStream, VersionFilter}, triple, AttributeNode, Attributes, Entity, EntityFilter, EntityNode, FromAttributes, Triple, Value
};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RelationEdge<T> {
    pub id: String,

    pub from: T,
    pub to: T,
    pub relation_type: String, // TODO: Change to T

    pub index: String,

    /// System properties
    #[serde(flatten)]
    pub system_properties: SystemProperties,
}

impl RelationEdge<EntityNodeRef> {
    pub fn new(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        relation_type: impl Into<String>,
        index: impl Into<Value>,
    ) -> Self {
        Self {
            id: id.into(),
            from: EntityNodeRef(from.into()),
            to: EntityNodeRef(to.into()),
            relation_type: relation_type.into(),
            index: Into::<Value>::into(index).value,
            system_properties: SystemProperties::default(),
        }
    }

    /// Create a new TYPES relation
    pub fn new_types(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        index: impl Into<Value>,
    ) -> Self {
        Self::new(id, from, to, system_ids::TYPES_ATTRIBUTE, index)
    }

    pub fn insert(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
    ) -> InsertOneQuery {
        InsertOneQuery::new(neo4j, block, space_id.into(), space_version.into(), self)
    }

    pub fn get_attributes(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: impl Into<String>,
        space_version: Option<String>,
    ) -> attributes::FindOneQuery {
        attributes::FindOneQuery::new(neo4j, self.id.clone(), space_id.into(), space_version)
    }

    pub fn set_attribute(
        &self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
        attribute: AttributeNode,
    ) -> triple::InsertOneQuery {
        triple::InsertOneQuery::new(
            neo4j,
            block,
            space_id.into(),
            space_version.into(),
            Triple {
                entity: self.id.clone(),
                attribute: attribute.id,
                value: attribute.value,
            },
        )
    }

    pub fn set_attributes(
        &self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
        attributes: Attributes,
    ) -> attributes::InsertOneQuery<Attributes> {
        attributes::InsertOneQuery::new(
            neo4j,
            block,
            self.id.clone(),
            space_id.into(),
            space_version.into(),
            attributes,
        )
    }

    pub fn to(&self, neo4j: &neo4rs::Graph) -> entity_node::FindOneQuery {
        entity_node::find_one(neo4j, &self.to)
    }

    pub fn from(&self, neo4j: &neo4rs::Graph) -> entity_node::FindOneQuery {
        entity_node::find_one(neo4j, &self.from)
    }

    pub fn relation_type(&self, neo4j: &neo4rs::Graph) -> entity_node::FindOneQuery {
        entity_node::find_one(neo4j, &self.relation_type)
    }

    pub fn entity(&self, neo4j: &neo4rs::Graph) -> entity_node::FindOneQuery {
        entity_node::find_one(neo4j, &self.id)
    }

    pub fn index(&self) -> &str {
        &self.index
    }
}

impl From<pb::ipfs::Relation> for RelationEdge<EntityNodeRef> {
    fn from(relation: pb::ipfs::Relation) -> Self {
        Self {
            id: relation.id,
            from: relation.from_entity.into(),
            to: relation.to_entity.into(),
            relation_type: relation.r#type.into(),
            index: relation.index,
            system_properties: SystemProperties::default(),
        }
    }
}

impl From<RelationEdge<EntityNodeRef>> for BoltType {
    fn from(relation: RelationEdge<EntityNodeRef>) -> Self {
        let mut triple_bolt_map = HashMap::new();
        triple_bolt_map.insert(
            neo4rs::BoltString { value: "id".into() },
            relation.id.into(),
        );
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "from".into(),
            },
            relation.from.0.into(),
        );
        triple_bolt_map.insert(
            neo4rs::BoltString { value: "to".into() },
            relation.to.0.into(),
        );
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "relation_type".into(),
            },
            relation.relation_type.into(),
        );
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "index".into(),
            },
            relation.index.into(),
        );

        BoltType::Map(neo4rs::BoltMap {
            value: triple_bolt_map,
        })
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

pub fn delete_many(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> DeleteManyQuery {
    DeleteManyQuery::new(neo4j, block, space_id.into(), space_version.into())
}

pub fn find_one<T>(
    neo4j: &neo4rs::Graph,
    relation_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
) -> FindOneQuery<T> {
    FindOneQuery::new(neo4j, relation_id.into(), space_id.into(), space_version)
}

pub fn find_many<T>(neo4j: &neo4rs::Graph) -> FindManyQuery<T> {
    FindManyQuery::new(neo4j)
}

pub fn find_one_to<T>(
    neo4j: &neo4rs::Graph,
    relation_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
) -> FindOneToQuery<T> {
    FindOneToQuery::new(neo4j, relation_id.into(), space_id.into(), space_version)
}

pub fn find_many_to<T>(neo4j: &neo4rs::Graph) -> FindManyToQuery<T> {
    FindManyToQuery::new(neo4j)
}

pub fn insert_one(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
    relation: RelationEdge<EntityNodeRef>,
) -> InsertOneQuery {
    InsertOneQuery::new(
        neo4j,
        block,
        space_id.into(),
        space_version.into(),
        relation,
    )
}

pub fn insert_many(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> InsertManyQuery {
    InsertManyQuery::new(neo4j, block, space_id.into(), space_version.into())
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
        // TODO: Add relation entity deletion
        const QUERY: &str = const_format::formatcp!(
            r#"
                MATCH () -[r:RELATION {{id: $relation_id}}]-> ()
                WHERE r.space_id = $space_id
                AND r.max_version IS NULL
                SET r.max_version = $space_version
                SET r += {{
                    `{UPDATED_AT}`: datetime($block_timestamp),
                    `{UPDATED_AT_BLOCK}`: $block_number
                }}
            "#,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("relation_id", self.relation_id)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("block_timestamp", self.block.timestamp.to_rfc3339())
            .param("block_number", self.block.block_number.to_string());

        Ok(self.neo4j.run(query).await?)
    }
}

pub struct DeleteManyQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: String,
    relations: Vec<String>,
}

impl DeleteManyQuery {
    fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: String,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            relations: vec![],
        }
    }

    pub fn relation(mut self, relation_id: impl Into<String>) -> Self {
        self.relations.push(relation_id.into());
        self
    }

    pub fn relation_mut(&mut self, relation_id: impl Into<String>) {
        self.relations.push(relation_id.into());
    }

    pub fn relations(mut self, relation_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.relations
            .extend(relation_ids.into_iter().map(Into::into));
        self
    }

    pub fn relations_mut(&mut self, relation_ids: impl IntoIterator<Item = impl Into<String>>) {
        self.relations
            .extend(relation_ids.into_iter().map(Into::into));
    }
}

impl Query<()> for DeleteManyQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        // TODO: Add relation entity deletion
        const QUERY: &str = const_format::formatcp!(
            r#"
                UNWIND $relations as relation_id
                MATCH () -[r:RELATION {{id: relation_id}}]-> ()
                WHERE r.space_id = $space_id
                AND r.max_version IS NULL
                SET r.max_version = $space_version
                SET r += {{
                    `{UPDATED_AT}`: datetime($block_timestamp),
                    `{UPDATED_AT_BLOCK}`: $block_number
                }}
            "#,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("relations", self.relations)
            .param("block_timestamp", self.block.timestamp.to_rfc3339())
            .param("block_number", self.block.block_number.to_string());

        Ok(self.neo4j.run(query).await?)
    }
}

pub struct InsertOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: String,
    relation: RelationEdge<EntityNodeRef>,
}

impl InsertOneQuery {
    fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: String,
        relation: RelationEdge<EntityNodeRef>,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            relation,
        }
    }
}

impl Query<()> for InsertOneQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        // TODO: Add old relation deletion
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (from:Entity {{id: $relation.from}})
            MATCH (to:Entity {{id: $relation.to}})
            CREATE (from) -[r:RELATION]-> (to)
            SET r += {{
                id: $relation.id,
                space_id: $space_id,
                index: $relation.index,
                min_version: $space_version,
                relation_type: $relation.relation_type,
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number,
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
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
            .param("relation", self.relation)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

pub struct InsertManyQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: String,
    relations: Vec<RelationEdge<EntityNodeRef>>,
}

impl InsertManyQuery {
    fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: String,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            relations: vec![],
        }
    }

    pub fn relation(mut self, relation: RelationEdge<EntityNodeRef>) -> Self {
        self.relations.push(relation);
        self
    }

    pub fn relation_mut(&mut self, relation: RelationEdge<EntityNodeRef>) {
        self.relations.push(relation);
    }

    pub fn relations(mut self, relations: impl IntoIterator<Item = RelationEdge<EntityNodeRef>>) -> Self {
        self.relations.extend(relations);
        self
    }

    pub fn relations_mut(&mut self, relations: impl IntoIterator<Item = RelationEdge<EntityNodeRef>>) {
        self.relations.extend(relations);
    }
}

impl Query<()> for InsertManyQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        if self.relations.is_empty() {
            return Ok(());
        }

        const QUERY: &str = const_format::formatcp!(
            r#"
            UNWIND $relations as relation
            MATCH (from:Entity {{id: relation.from}})
            MATCH (to:Entity {{id: relation.to}})
            CREATE (from) -[r:RELATION]-> (to)
            SET r += {{
                id: relation.id,
                space_id: $space_id,
                index: relation.index,
                min_version: $space_version,
                relation_type: relation.relation_type,
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number,
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!("relation_node::InsertManyQuery:\n{}", QUERY);
            tracing::info!("relations:\n{:#?}", self.relations);
        };

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("relations", self.relations)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

pub struct FindOneQuery<T> {
    neo4j: neo4rs::Graph,
    id: String,
    space_id: String,
    space_version: VersionFilter,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> FindOneQuery<T> {
    fn new(
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

    pub fn select_to<U>(self) -> FindOneToQuery<U> {
        FindOneToQuery {
            neo4j: self.neo4j,
            id: self.id,
            space_id: self.space_id,
            space_version: self.space_version,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Query<Option<RelationEdge<EntityNodeRef>>> for FindOneQuery<EntityNodeRef> {
    async fn send(self) -> Result<Option<RelationEdge<EntityNodeRef>>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query = QueryPart::default()
            .match_clause(
                "(from:Entity) -[r:RELATION {id: $id, space_id: $space_id}]-> (to:Entity)",
            )
            .merge(self.space_version.into_query_part("r"))
            .return_clause("r{.*, from: from.id, to: to.id} as r")
            .order_by_clause("r.index")
            .params("id", self.id)
            .params("space_id", self.space_id)
            .build();

        neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| Result::<_, DatabaseError>::Ok(row.to::<RelationEdge<EntityNodeRef>>()?))
            .transpose()
    }
}

impl Query<Option<RelationEdge<EntityNode>>> for FindOneQuery<EntityNode> {
    async fn send(self) -> Result<Option<RelationEdge<EntityNode>>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query = QueryPart::default()
            .match_clause(
                "(from:Entity) -[r:RELATION {id: $id, space_id: $space_id}]-> (to:Entity)",
            )
            .merge(self.space_version.into_query_part("r"))
            .return_clause("r{.*, from: from, to: to} as r")
            .order_by_clause("r.index")
            .params("id", self.id)
            .params("space_id", self.space_id)
            .build();

        neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| Result::<_, DatabaseError>::Ok(row.to::<RelationEdge<EntityNode>>()?))
            .transpose()
    }
}

pub struct FindOneToQuery<T> {
    neo4j: neo4rs::Graph,
    id: String,
    space_id: String,
    space_version: VersionFilter,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> FindOneToQuery<T> {
    fn new(
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
            .merge(self.space_version.into_query_part("r"))
            .return_clause("to")
            .limit(1)
            .order_by_clause("r.index")
            .params("id", self.id)
            .params("space_id", self.space_id)
            .build();

        #[derive(Debug, Deserialize)]
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
        let neo4j = self.neo4j.clone();

        let match_relation = MatchOneRelation::new(
            self.id.clone(),
            self.space_id.clone(),
            &self.space_version,
        );

        let space_filter = Some(prop_filter::value(self.space_id.clone()));
        let match_entity = MatchEntity::new(
            &space_filter,
            &self.space_version
        );

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

        #[derive(Debug, Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            node: EntityNode,
            attrs: Vec<AttributeNode>,
            types: Vec<EntityNode>,
        }

        neo4j
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


pub struct FindManyQuery<T> {
    neo4j: neo4rs::Graph,
    id: Option<PropFilter<String>>,
    from_id: Option<PropFilter<String>>,
    to_id: Option<PropFilter<String>>,
    relation_type: Option<PropFilter<String>>,

    from_: Option<EntityFilter>,
    to_: Option<EntityFilter>,

    space_id: Option<PropFilter<String>>,
    space_version: VersionFilter,

    limit: usize,
    skip: Option<usize>,

    _phantom: std::marker::PhantomData<T>,
}

impl<T> FindManyQuery<T> {
    pub fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id: None,
            relation_type: None,
            from_id: None,
            to_id: None,
            from_: None,
            to_: None,
            space_id: None,
            space_version: VersionFilter::default(),
            limit: 100,
            skip: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn select_to(self) -> FindManyToQuery<T> {
        FindManyToQuery {
            neo4j: self.neo4j,
            id: self.id,
            from_id: self.from_id,
            to_id: self.to_id,
            relation_type: self.relation_type,
            from_: self.from_,
            to_: self.to_,
            space_id: self.space_id,
            space_version: self.space_version,
            limit: self.limit,
            skip: self.skip,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn id(mut self, id: PropFilter<String>) -> Self {
        self.id = Some(id);
        self
    }

    pub fn from_id(mut self, from_id: PropFilter<String>) -> Self {
        self.from_id = Some(from_id);
        self
    }

    pub fn to_id(mut self, to_id: PropFilter<String>) -> Self {
        self.to_id = Some(to_id);
        self
    }

    pub fn relation_type(mut self, relation_type: PropFilter<String>) -> Self {
        self.relation_type = Some(relation_type);
        self
    }

    pub fn from_(mut self, from_: EntityFilter) -> Self {
        self.from_ = Some(from_);
        self
    }

    pub fn to_(mut self, to_: EntityFilter) -> Self {
        self.to_ = Some(to_);
        self
    }

    pub fn space_id(mut self, space_id: PropFilter<String>) -> Self {
        self.space_id = Some(space_id);
        self
    }

    pub fn version(mut self, space_version: Option<String>) -> Self {
        if let Some(space_version) = space_version {
            self.space_version.version_mut(space_version);
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
            .merge(self.space_version.into_query_part("r"))
            .order_by_clause("r.index")
            .limit(self.limit);

        if let Some(id_filter) = self.id {
            query_part.merge_mut(id_filter.into_query_part("r", "id", None));
        }

        if let Some(from_id) = self.from_id {
            query_part = query_part.merge(from_id.into_query_part("from", "id", None));
        }

        if let Some(to_id) = self.to_id {
            query_part = query_part.merge(to_id.into_query_part("to", "id", None));
        }

        if let Some(relation_type) = self.relation_type {
            query_part =
                query_part.merge(relation_type.into_query_part("r", "relation_type", None));
        }

        if let Some(from_filter) = self.from_ {
            query_part = query_part.merge(from_filter.into_query_part("from"));
        }

        if let Some(to_filter) = self.to_ {
            query_part = query_part.merge(to_filter.into_query_part("to"));
        }

        if let Some(space_id) = self.space_id {
            query_part = query_part.merge(space_id.into_query_part("r", "space_id", None));
        }

        if let Some(skip) = self.skip {
            query_part = query_part.skip(skip);
        }

        query_part
    }
}

impl QueryStream<RelationEdge<EntityNodeRef>> for FindManyQuery<EntityNodeRef> {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<RelationEdge<EntityNodeRef>, DatabaseError>>, DatabaseError>
    {
        let neo4j = self.neo4j.clone();
        let query_part = self.into_query_part()
            .return_clause("r{.*, from: from.id, to: to.id} as r");

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!("relation_node::FindManyQuery:\n{}", query_part);
        };
        let query = query_part.build();

        Ok(neo4j
            .execute(query)
            .await?
            .into_stream_as::<RelationEdge<EntityNodeRef>>()
            .map_err(DatabaseError::from))
    }
}

impl QueryStream<RelationEdge<EntityNode>> for FindManyQuery<EntityNode> {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<RelationEdge<EntityNode>, DatabaseError>>, DatabaseError>
    {
        let neo4j = self.neo4j.clone();
        let query_part = self.into_query_part()
            .return_clause("r{.*, from: from, to: to} as r");

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!("relation_node::FindManyQuery:\n{}", query_part);
        };
        let query = query_part.build();

        Ok(neo4j
            .execute(query)
            .await?
            .into_stream_as::<RelationEdge<EntityNode>>()
            .map_err(DatabaseError::from))
    }
}

pub struct FindManyToQuery<T> {
    neo4j: neo4rs::Graph,
    id: Option<PropFilter<String>>,
    from_id: Option<PropFilter<String>>,
    to_id: Option<PropFilter<String>>,
    relation_type: Option<PropFilter<String>>,

    from_: Option<EntityFilter>,
    to_: Option<EntityFilter>,

    space_id: Option<PropFilter<String>>,
    space_version: VersionFilter,

    limit: usize,
    skip: Option<usize>,

    _phantom: std::marker::PhantomData<T>,
}

impl<T> FindManyToQuery<T> {
    fn into_query_part(&self) -> QueryPart {
        let mut query_part = QueryPart::default()
            .match_clause("(from:Entity) -[r:RELATION]-> (to:Entity)")
            .merge(self.space_version.into_query_part("r"))
            .order_by_clause("r.index")
            .limit(self.limit);

        if let Some(id_filter) = &self.id {
            query_part.merge_mut(id_filter.into_query_part("r", "id", None));
        }

        if let Some(from_id) = &self.from_id {
            query_part = query_part.merge(from_id.into_query_part("from", "id", None));
        }

        if let Some(to_id) = &self.to_id {
            query_part = query_part.merge(to_id.into_query_part("to", "id", None));
        }

        if let Some(relation_type) = &self.relation_type {
            query_part =
                query_part.merge(relation_type.into_query_part("r", "relation_type", None));
        }

        if let Some(from_filter) = &self.from_ {
            query_part = query_part.merge(from_filter.into_query_part("from"));
        }

        if let Some(to_filter) = &self.to_ {
            query_part = query_part.merge(to_filter.into_query_part("to"));
        }

        if let Some(space_id) = &self.space_id {
            query_part = query_part.merge(space_id.into_query_part("r", "space_id", None));
        }

        if let Some(skip) = self.skip {
            query_part = query_part.skip(skip);
        }

        query_part
    }
}

impl QueryStream<EntityNode> for FindManyToQuery<EntityNode> {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<EntityNode, DatabaseError>>, DatabaseError>
    {
        let neo4j = self.neo4j.clone();
        let query_part = self.into_query_part()
            .return_clause("to");

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
    ) -> Result<impl Stream<Item = Result<Entity<T>, DatabaseError>>, DatabaseError>
    {
        let neo4j = self.neo4j.clone();

        let match_entity = MatchEntity::new(
            &self.space_id,
            &self.space_version
        );

        let query_part = self.into_query_part()
            .with_clause("to", match_entity.chain(
                "to", 
                "attrs", 
                "types", 
                query_part::return_query("to{.*, attrs: attrs, types: types}"),
            ));

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!("relation_node::FindManyToQuery:\n{}", query_part);
        };
        let query = query_part.build();

        #[derive(Debug, Deserialize)]
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

impl<T> FindManyToQuery<T> {
    pub fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id: None,
            relation_type: None,
            from_id: None,
            to_id: None,
            from_: None,
            to_: None,
            space_id: None,
            space_version: VersionFilter::default(),
            limit: 100,
            skip: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{entity, mapping};

    use super::*;

    use testcontainers::{
        core::{IntoContainerPort, WaitFor},
        runners::AsyncRunner,
        GenericImage, ImageExt,
    };

    const BOLT_PORT: u16 = 7687;
    const HTTP_PORT: u16 = 7474;

    #[tokio::test]
    async fn test_find_one() {
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

        let block = &BlockMetadata::default();

        neo4j.run(neo4rs::query(&format!(
            r#"
            CREATE (alice:Entity {{id: "alice"}})
            CREATE (bob:Entity {{id: "bob"}})
            CREATE (knows:Entity {{id: "knows"}})
            CREATE (alice) -[r:RELATION {{id: "abc", relation_type: "knows", space_id: "ROOT", min_version: "0", index: "0"}}]-> (bob)
            SET r += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number,
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        ))
            .param("block_timestamp", block.timestamp.to_rfc3339())
            .param("block_number", block.block_number.to_string())
        )
            .await
            .expect("Failed to insert data");

        let relation_node = RelationEdge::new("abc", "alice", "bob", "knows", "0");

        let found_relation = find_one::<EntityNodeRef>(&neo4j, "abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find relation")
            .expect("Relation not found");

        assert_eq!(found_relation, relation_node);
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

        triple::insert_many(&neo4j, &BlockMetadata::default(), "space_id", "0")
            .triples(vec![
                Triple::new("alice", "name", "Alice"),
                Triple::new("bob", "name", "Bob"),
                Triple::new("knows", "name", "knows"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation_node = RelationEdge::new("abc", "alice", "bob", "knows", "0");

        relation_node
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let found_relation: RelationEdge<EntityNodeRef> = find_one::<EntityNodeRef>(&neo4j, "abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find relation")
            .expect("Relation not found");

        assert_eq!(found_relation, relation_node);
    }

    #[tokio::test]
    async fn test_insert_find_one_node() {
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

        triple::insert_many(&neo4j, &BlockMetadata::default(), "space_id", "0")
            .triples(vec![
                Triple::new("alice", "name", "Alice"),
                Triple::new("bob", "name", "Bob"),
                Triple::new("knows", "name", "knows"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation_node = RelationEdge::new("abc", "alice", "bob", "knows", "0");

        relation_node
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let found_relation = find_one::<EntityNode>(&neo4j, "abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find relation")
            .expect("Relation not found");

        assert_eq!(
            found_relation,
            RelationEdge {
                id: "abc".to_string(),
                from: EntityNode {
                    id: "alice".to_string(),
                    system_properties: BlockMetadata::default().into(),
                },
                to: EntityNode {
                    id: "bob".to_string(),
                    system_properties: BlockMetadata::default().into(),
                },
                relation_type: "knows".to_string(),
                index: "0".to_string(),
                system_properties: BlockMetadata::default().into(),
            },
        );
    }

    #[tokio::test]
    async fn test_find_many() {
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

        let block = &BlockMetadata::default();

        neo4j.run(neo4rs::query(&format!(
            r#"
            CREATE (alice:Entity {{id: "alice"}})
            CREATE (bob:Entity {{id: "bob"}})
            CREATE (charlie:Entity {{id: "charlie"}})
            CREATE (knows:Entity {{id: "knows"}})
            CREATE (alice) -[r1:RELATION {{id: "abc", relation_type: "knows", space_id: "ROOT", min_version: "0", index: "0"}}]-> (bob)
            SET r1 += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number,
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            CREATE (alice) -[r2:RELATION {{id: "dev", relation_type: "knows", space_id: "ROOT", min_version: "0", index: "1"}}]-> (charlie)
            SET r2 += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number,
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        ))
            .param("block_timestamp", block.timestamp.to_rfc3339())
            .param("block_number", block.block_number.to_string())
        )
            .await
            .expect("Failed to insert data");

        let relation_nodes = vec![
            RelationEdge::new("abc", "alice", "bob", "knows", "0"),
            RelationEdge::new("dev", "alice", "charlie", "knows", "1"),
        ];

        let found_relations = find_many::<EntityNodeRef>(&neo4j)
            .relation_type(PropFilter::default().value("knows"))
            .from_id(PropFilter::default().value("alice"))
            .send()
            .await
            .expect("Failed to find relations")
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to collect triples");

        assert_eq!(found_relations, relation_nodes);
    }

    #[tokio::test]
    async fn test_insert_many_find_many() {
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

        let relation_nodes = vec![
            RelationEdge::new("abc", "alice", "bob", "knows", "0"),
            RelationEdge::new("dev", "alice", "charlie", "knows", "1"),
        ];

        insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .relations(relation_nodes.clone())
            .send()
            .await
            .expect("Failed to insert relations");

        let found_relations = find_many::<EntityNodeRef>(&neo4j)
            .relation_type(PropFilter::default().value("knows"))
            .from_id(PropFilter::default().value("alice"))
            .send()
            .await
            .expect("Failed to find relations")
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to collect triples");

        assert_eq!(found_relations, relation_nodes);
    }

    #[tokio::test]
    async fn test_insert_many_find_many_node() {
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

        let relation_nodes = vec![
            RelationEdge::new("abc", "alice", "bob", "knows", "0"),
            RelationEdge::new("dev", "alice", "charlie", "knows", "1"),
        ];

        insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .relations(relation_nodes.clone())
            .send()
            .await
            .expect("Failed to insert relations");

        let found_relations = find_many::<EntityNode>(&neo4j)
            .relation_type(PropFilter::default().value("knows"))
            .from_id(PropFilter::default().value("alice"))
            .send()
            .await
            .expect("Failed to find relations")
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to collect triples");

        assert_eq!(
            found_relations, 
            vec![
                RelationEdge {
                    id: "abc".to_string(),
                    from: EntityNode {
                        id: "alice".to_string(),
                        system_properties: BlockMetadata::default().into(),
                    },
                    to: EntityNode {
                        id: "bob".to_string(),
                        system_properties: BlockMetadata::default().into(),
                    },
                    relation_type: "knows".to_string(),
                    index: "0".to_string(),
                    system_properties: BlockMetadata::default().into(),
                },
                RelationEdge {
                    id: "dev".to_string(),
                    from: EntityNode {
                        id: "alice".to_string(),
                        system_properties: BlockMetadata::default().into(),
                    },
                    to: EntityNode {
                        id: "charlie".to_string(),
                        system_properties: BlockMetadata::default().into(),
                    },
                    relation_type: "knows".to_string(),
                    index: "1".to_string(),
                    system_properties: BlockMetadata::default().into(),
                }
            ],
        );
    }

    #[tokio::test]
    async fn test_find_one_to_node() {
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
        #[derive(Clone, Debug, PartialEq)]
        struct Person {
            name: String,
        }

        impl mapping::IntoAttributes for Person {
            fn into_attributes(self) -> Result<mapping::Attributes, mapping::TriplesConversionError> {
                Ok(mapping::Attributes::default()
                    .attribute(("name", self.name)))
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

        triple::insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .triples(vec![
                Triple::new("person_type", "name", "Person"),
                Triple::new("knows", "name", "knows"),
                Triple::new(system_ids::TYPES_ATTRIBUTE, "name", "Types"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        Entity::new("alice", Person {name: "Alice".to_string()})
            .with_type("person_type")
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert entity");

        Entity::new("bob", Person {name: "Bob".to_string()})
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
                attributes: Person { name: "Bob".to_string() },
                types: vec!["person_type".to_string()],
            },
        );
    }

    #[tokio::test]
    async fn test_find_many_to_entity() {
        #[derive(Clone, Debug, PartialEq)]
        struct Person {
            name: String,
        }

        impl mapping::IntoAttributes for Person {
            fn into_attributes(self) -> Result<mapping::Attributes, mapping::TriplesConversionError> {
                Ok(mapping::Attributes::default()
                    .attribute(("name", self.name)))
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

        triple::insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .triples(vec![
                Triple::new("person_type", "name", "Person"),
                Triple::new("knows", "name", "knows"),
                Triple::new(system_ids::TYPES_ATTRIBUTE, "name", "Types"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        Entity::<Person>::new("alice", Person {name: "Alice".to_string()})
            .with_type("person_type")
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert entity");

        Entity::<Person>::new("bob", Person {name: "Bob".to_string()})
            .with_type("person_type")
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert entity");

        Entity::<Person>::new("charlie", Person {name: "Charlie".to_string()})
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

        let found_relations = find_many::<Entity<Person>>(&neo4j)
            .relation_type(PropFilter::default().value("knows"))
            .from_id(PropFilter::default().value("alice"))
            .select_to()
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
                    attributes: Person { name: "Bob".to_string() },
                    types: vec!["person_type".to_string()],
                },
                Entity {
                    node: EntityNode {
                        id: "charlie".to_string(),
                        system_properties: BlockMetadata::default().into(),
                    },
                    attributes: Person { name: "Charlie".to_string() },
                    types: vec!["person_type".to_string()],
                }
            ],
        );
    }
}
