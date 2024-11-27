use std::collections::HashMap;

use kg_core::system_ids;

pub struct Relation<T> {
    pub id: String,
    pub from: String,
    pub to: String,
    pub relation_type: String,
    pub(crate) data: T,
}

impl<T> Relation<T> {
    pub fn new(id: &str, from: &str, to: &str, relation_type: &str, data: T) -> Self {
        Self {
            id: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            relation_type: relation_type.to_string(),
            data,
        }
    }
}

impl Relation<HashMap<String, neo4rs::BoltType>> {
    pub fn with_attribute<T>(mut self, key: String, value: T) -> Self
    where
        T: Into<neo4rs::BoltType>,
    {
        self.data.insert(key, value.into());
        self
    }
}

pub struct Node<T> {
    pub id: String,
    pub types: Vec<String>,
    pub(crate) data: T,
}

impl<T> Node<T> {
    pub fn new(id: String, data: T) -> Self {
        Self {
            id,
            types: Vec::new(),
            data,
        }
    }

    pub fn with_type(mut self, type_id: &str) -> Self {
        self.types.push(type_id.to_string());
        self
    }
}

impl Node<HashMap<String, neo4rs::BoltType>> {
    pub fn with_attribute<T>(mut self, attribute_id: String, value: T) -> Self
    where
        T: Into<neo4rs::BoltType>,
    {
        self.data.insert(attribute_id, value.into());
        self
    }
}