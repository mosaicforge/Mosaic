use std::collections::HashMap;

use neo4rs::BoltType;
use serde::Deserialize;

use crate::{error::DatabaseError, indexer_ids, models::BlockMetadata, system_ids};

use super::{
    attributes, entity_node,
    query_utils::{PropFilter, Query, QueryPart, VersionFilter},
    triple, AttributeNode, Attributes, Triple, Value,
};

#[derive(Clone, Debug, Deserialize)]
pub struct RelationNode {
    pub id: String,

    pub from: String,
    pub to: String,
    pub relation_type: String,
    pub index: Value,
}

impl RelationNode {
    pub fn new(id: &str, from: &str, to: &str, relation_type: &str, index: Value) -> Self {
        Self {
            id: id.to_owned(),
            from: from.to_owned(),
            to: to.to_owned(),
            relation_type: relation_type.to_owned(),
            index,
        }
    }

    /// Create a new TYPES relation
    pub fn new_types(id: &str, from: &str, to: &str, index: Value) -> Self {
        Self::new(id, from, to, system_ids::TYPES_ATTRIBUTE, index)
    }

    pub fn insert(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: i64,
    ) -> InsertOneQuery {
        InsertOneQuery::new(neo4j, block, space_id.into(), space_version, self)
    }

    pub fn get_attributes(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: &str,
        space_version: Option<i64>,
    ) -> attributes::FindOneQuery {
        attributes::FindOneQuery::new(neo4j, self.id.clone(), space_id.to_owned(), space_version)
    }

    pub fn set_attribute(
        &self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: &str,
        space_version: i64,
        attribute: AttributeNode,
    ) -> triple::InsertOneQuery {
        triple::InsertOneQuery::new(
            neo4j,
            block,
            space_id.to_owned(),
            space_version,
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
        space_id: &str,
        space_version: i64,
        attributes: Attributes,
    ) -> attributes::InsertOneQuery<Attributes> {
        attributes::InsertOneQuery::new(
            neo4j,
            block,
            self.id.clone(),
            space_id.to_owned(),
            space_version,
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
}

impl Into<BoltType> for RelationNode {
    fn into(self) -> BoltType {
        let mut triple_bolt_map = HashMap::new();
        triple_bolt_map.insert(neo4rs::BoltString { value: "id".into() }, self.id.into());
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "from".into(),
            },
            self.from.into(),
        );
        triple_bolt_map.insert(neo4rs::BoltString { value: "to".into() }, self.to.into());
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "relation_type".into(),
            },
            self.relation_type.into(),
        );
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "index".into(),
            },
            self.index.into(),
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
    space_version: i64,
) -> DeleteOneQuery {
    DeleteOneQuery::new(
        neo4j.clone(),
        block.clone(),
        relation_id.into(),
        space_id.into(),
        space_version,
    )
}

pub fn insert_one(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: i64,
    relation: RelationNode,
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
    space_version: i64,
    relations: Vec<RelationNode>,
) -> InsertManyQuery {
    InsertManyQuery::new(neo4j, block, space_id.into(), space_version, relations)
}

pub fn find_one(
    neo4j: &neo4rs::Graph,
    relation_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<i64>,
) -> FindOneQuery {
    FindOneQuery::new(neo4j, relation_id.into(), space_id.into(), space_version)
}

pub fn find_many(neo4j: &neo4rs::Graph) -> FindManyQuery {
    FindManyQuery::new(neo4j)
}

pub struct DeleteOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    relation_id: String,
    space_id: String,
    space_version: i64,
}

impl DeleteOneQuery {
    pub fn new(
        neo4j: neo4rs::Graph,
        block: BlockMetadata,
        relation_id: String,
        space_id: String,
        space_version: i64,
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
        const QUERY: &str = const_format::formatcp!(
            r#"
                MATCH (r {{id: $relation_id}})
                MATCH (r)-[r_to:`{TO_ENTITY}` {{space_id: $space_id, max_version: null}}]->()
                MATCH (r)-[r_from:`{FROM_ENTITY}` {{space_id: $space_id, max_version: null}}]->()
                MATCH (r)-[r_rt:`{RELATION_TYPE}` {{space_id: $space_id, max_version: null}}]->()
                MATCH (r)-[r_index:ATTRIBUTE {{space_id: $space_id, max_version: null}}]->({{id: "{INDEX}"}})
                SET r_to.max_version = $space_version
                SET r_from.max_version = $space_version
                SET r_rt.max_version = $space_version
                SET r_index.max_version = $space_version
                SET r += {{
                    `{UPDATED_AT}`: datetime($block_timestamp),
                    `{UPDATED_AT_BLOCK}`: $block_number
                }}
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
            INDEX = system_ids::RELATION_INDEX,
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

pub struct InsertOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: i64,
    relation: RelationNode,
}

impl InsertOneQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: i64,
        relation: RelationNode,
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
        const QUERY: &str = const_format::formatcp!(
            r#"
            MERGE (e {{id: $relation.id}})
            ON CREATE SET e += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number
            }}
            ON MATCH CALL (e) {{
                MATCH (e) -[r_from:`{FROM_ENTITY}` {{space_id: $space_id, max_version: null}}]-> ()
                MATCH (e) -[r_to:`{TO_ENTITY}` {{space_id: $space_id, max_version: null}}]-> ()
                MATCH (e) -[r_rt:`{RELATION_TYPE}` {{space_id: $space_id, max_version: null}}]-> ()
                MATCH (e) -[r_index:ATTRIBUTE {{space_id: $space_id, max_version: null}}]-> (index {{id: "{INDEX}}})
                SET r_from.max_version = $space_version
                SET r_to.max_version = $space_version
                SET r_rt.max_version = $space_version
                SET r_index.max_version = $space_version
            }}
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            MATCH (from {{id: $relation.from}})
            MATCH (to {{id: $relation.to}})
            MATCH (rt {{id: $relation.relation_type}})
            CREATE (e) -[:`{FROM_ENTITY}` {{space_id: $space_id, min_version: $space_version}}]-> (from)
            CREATE (e) -[:`{TO_ENTITY}` {{space_id: $space_id, min_version: $space_version}}]-> (to)
            CREATE (e) -[:`{RELATION_TYPE}` {{space_id: $space_id, min_version: $space_version}}]-> (rt)
            CREATE (e) -[:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (index {{id: "{INDEX}}})
            SET index += $relation.index
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
            INDEX = system_ids::RELATION_INDEX,
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
    space_version: i64,
    relations: Vec<RelationNode>,
}

impl InsertManyQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: i64,
        relations: Vec<RelationNode>,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            relations,
        }
    }
}

impl Query<()> for InsertManyQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            UNWIND $relations as relation
            MERGE (e {{id: relation.id}})
            ON CREATE SET e += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number
            }}
            ON MATCH CALL (e) {{
                MATCH (e) -[r_from:`{FROM_ENTITY}` {{space_id: $space_id, max_version: null}}]-> ()
                MATCH (e) -[r_to:`{TO_ENTITY}` {{space_id: $space_id, max_version: null}}]-> ()
                MATCH (e) -[r_rt:`{RELATION_TYPE}` {{space_id: $space_id, max_version: null}}]-> ()
                MATCH (e) -[r_index:ATTRIBUTE {{space_id: $space_id, max_version: null}}]-> (index {{attribute: "{INDEX}}})
                SET r_from.max_version = $space_version
                SET r_to.max_version = $space_version
                SET r_rt.max_version = $space_version
                SET r_index.max_version = $space_version
            }}
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            MATCH (from {{id: relation.from}})
            MATCH (to {{id: relation.to}})
            MATCH (rt {{id: relation.relation_type}})
            CREATE (e) -[:`{FROM_ENTITY}` {{space_id: $space_id, min_version: $space_version}}]-> (from)
            CREATE (e) -[:`{TO_ENTITY}` {{space_id: $space_id, min_version: $space_version}}]-> (to)
            CREATE (e) -[:`{RELATION_TYPE}` {{space_id: $space_id, min_version: $space_version}}]-> (rt)
            CREATE (e) -[:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (index {{attribute: "{INDEX}}})
            SET index += relation.index
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
            INDEX = system_ids::RELATION_INDEX,
        );

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

pub struct FindOneQuery {
    neo4j: neo4rs::Graph,
    id: String,
    space_id: String,
    space_version: VersionFilter,
}

impl FindOneQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        id: String,
        space_id: String,
        space_version: Option<i64>,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id,
            space_id,
            space_version: VersionFilter::new(space_version),
        }
    }

    pub fn into_query_part(self) -> QueryPart {
        QueryPart::default()
            .match_clause("(e:Entity {id: $id})")
            .match_clause(format!(
                "(e) -[r_from:`{}` {{space_id: $space_id}}]-> (from)",
                system_ids::RELATION_FROM_ATTRIBUTE
            ))
            .match_clause(format!(
                "(e) -[r_to:`{}` {{space_id: $space_id}}]-> (to)",
                system_ids::RELATION_TO_ATTRIBUTE
            ))
            .match_clause(format!(
                "(e) -[r_rt:`{}` {{space_id: $space_id}}]-> (rt)",
                system_ids::RELATION_TYPE_ATTRIBUTE
            ))
            .match_clause(format!(
                r#"(e) -[r_index:ATTRIBUTE {{space_id: $space_id}}]-> (index {{attribute: "{}"}})"#,
                system_ids::RELATION_INDEX
            ))
            .merge(self.space_version.clone().into_query_part("r_from"))
            .merge(self.space_version.clone().into_query_part("r_to"))
            .merge(self.space_version.clone().into_query_part("r_rt"))
            .merge(self.space_version.into_query_part("r_index"))
            .return_clause(
                "e{{.id, from: from.id, to: to.id, relation_type: rt.id, index: index.value}}",
            )
            .order_by_clause("index.value")
            .params("space_id", self.space_id)
    }
}

impl Query<Option<RelationNode>> for FindOneQuery {
    async fn send(self) -> Result<Option<RelationNode>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query = self.into_query_part().build();

        #[derive(Debug, Deserialize)]
        struct RowResult {
            n: RelationNode,
        }

        Ok(neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                Result::<_, DatabaseError>::Ok(row.n)
            })
            .transpose()?)
    }
}

pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    space_id: Option<PropFilter<String>>,
    relation_type: Option<PropFilter<String>>,
    from_id: Option<PropFilter<String>>,
    to_id: Option<PropFilter<String>>,

    space_version: VersionFilter,
}

impl FindManyQuery {
    pub fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            space_id: None,
            relation_type: None,
            from_id: None,
            to_id: None,
            space_version: VersionFilter::default(),
        }
    }
}

impl Query<Vec<RelationNode>> for FindManyQuery {
    async fn send(self) -> Result<Vec<RelationNode>, DatabaseError> {
        // let neo4j = self.neo4j.clone();
        // let query = self.into_query_part().build();

        // #[derive(Debug, Deserialize)]
        // struct RowResult {
        //     n: Relation,
        // }

        // Ok(neo4j
        //     .execute(query)
        //     .await?
        //     .into_stream_as::<RowResult>()
        //     .map_err(DatabaseError::from)
        //     .and_then(|row| async move {
        //         Ok(row.n)
        //     })
        //     .try_collect::<Vec<_>>()
        //     .await)
        todo!()
    }
}
