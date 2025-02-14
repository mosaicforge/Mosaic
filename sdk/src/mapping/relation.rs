use crate::{error::DatabaseError, models::BlockMetadata};

use super::{
    attributes::{self, IntoAttributes},
    entity_node,
    query_utils::Query,
    relation_node, RelationNode, Value,
};

/// High level model encapsulating a relation and its attributes.
pub struct Relation<T> {
    relation: RelationNode,

    pub attributes: T,
    pub types: Vec<String>,
}

impl<T> Relation<T> {
    pub fn new(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        relation_type: impl Into<String>,
        index: impl Into<Value>,
        attributes: T,
    ) -> Self {
        Relation {
            relation: RelationNode::new(id, from, to, relation_type, index),
            attributes,
            types: vec![],
        }
    }

    pub fn with_type(mut self, r#type: String) -> Self {
        self.types.push(r#type);
        self
    }

    pub fn insert(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: i64,
    ) -> InsertOneQuery<T> {
        InsertOneQuery::new(
            neo4j.clone(),
            block.clone(),
            space_id.into(),
            space_version,
            self,
        )
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

pub struct DeleteOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    relation_id: String,
    space_id: String,
    space_version: i64,
}

impl DeleteOneQuery {
    fn new(
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
        entity_node::delete_one(
            &self.neo4j,
            &self.block,
            &self.relation_id,
            &self.space_id,
            self.space_version,
        )
        .send()
        .await?;

        relation_node::delete_one(
            &self.neo4j,
            &self.block,
            &self.relation_id,
            &self.space_id,
            self.space_version,
        )
        .send()
        .await
    }
}

pub struct InsertOneQuery<T> {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    relation: Relation<T>,
    space_id: String,
    space_version: i64,
}

impl<T> InsertOneQuery<T> {
    fn new(
        neo4j: neo4rs::Graph,
        block: BlockMetadata,
        space_id: String,
        space_version: i64,
        relation: Relation<T>,
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
        let rel_id = self.relation.relation.id.clone();

        // Insert the relation node
        relation_node::insert_one(
            &self.neo4j,
            &self.block,
            &self.space_id,
            self.space_version,
            self.relation.relation,
        )
        .send()
        .await?;

        // Insert the relation attributes
        attributes::insert_one(
            &self.neo4j,
            &self.block,
            rel_id,
            &self.space_id,
            self.space_version,
            self.relation.attributes,
        )
        .send()
        .await
    }
}
