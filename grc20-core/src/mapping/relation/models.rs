use std::collections::HashMap;

use neo4rs::BoltType;

use crate::{
    block::BlockMetadata,
    mapping::{
        attributes,
        entity::{self, EntityNodeRef, SystemProperties},
        triple, AttributeNode, Attributes, Triple, Value,
    },
    pb, system_ids,
};

use super::InsertOneQuery;

/// Lightweight representation of a relation edge without it's properties.
#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
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
    ) -> InsertOneQuery<Self> {
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

    pub fn to<T>(&self, neo4j: &neo4rs::Graph) -> entity::FindOneQuery<T> {
        entity::find_one(neo4j, &self.to)
    }

    pub fn from<T>(&self, neo4j: &neo4rs::Graph) -> entity::FindOneQuery<T> {
        entity::find_one(neo4j, &self.from)
    }

    pub fn relation_type<T>(&self, neo4j: &neo4rs::Graph) -> entity::FindOneQuery<T> {
        entity::find_one(neo4j, &self.relation_type)
    }

    pub fn entity<T>(&self, neo4j: &neo4rs::Graph) -> entity::FindOneQuery<T> {
        entity::find_one(neo4j, &self.id)
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
            relation_type: relation.r#type,
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

/// High level model encapsulating a relation and its attributes.
#[derive(Clone, Debug, PartialEq)]
pub struct Relation<T, N> {
    pub(super) relation: RelationEdge<N>,

    pub attributes: T,
}

impl<T> Relation<T, EntityNodeRef> {
    pub fn new(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        relation_type: impl Into<String>,
        index: impl Into<Value>,
        attributes: T,
    ) -> Self {
        Relation {
            relation: RelationEdge::new(id, from, to, relation_type, index),
            attributes,
        }
    }

    pub fn insert(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: impl Into<String>,
    ) -> InsertOneQuery<Self> {
        InsertOneQuery::new(neo4j, block, space_id.into(), space_version.into(), self)
    }
}
