use std::collections::HashMap;

use neo4rs::BoltType;
use serde::Deserialize;

use super::{Triple, Value};

/// Neo4j model of an entity Attribute
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AttributeNode {
    pub(crate) id: String,

    #[serde(flatten)]
    pub(crate) value: Value,
}

impl AttributeNode {
    pub fn new(id: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
        }
    }
}

impl Into<BoltType> for AttributeNode {
    fn into(self) -> BoltType {
        let mut map = HashMap::new();
        map.insert(neo4rs::BoltString { value: "id".into() }, self.id.into());
        map.insert(
            neo4rs::BoltString {
                value: "value".into(),
            },
            self.value.value.into(),
        );
        map.insert(
            neo4rs::BoltString {
                value: "value_type".into(),
            },
            self.value.value_type.to_string().into(),
        );
        if let Some(format) = self.value.options.format {
            map.insert(
                neo4rs::BoltString {
                    value: "format".into(),
                },
                format.into(),
            );
        }
        if let Some(unit) = self.value.options.unit {
            map.insert(
                neo4rs::BoltString {
                    value: "unit".into(),
                },
                unit.into(),
            );
        }
        if let Some(language) = self.value.options.language {
            map.insert(
                neo4rs::BoltString {
                    value: "language".into(),
                },
                language.into(),
            );
        }

        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}

impl From<Triple> for AttributeNode {
    fn from(triple: Triple) -> Self {
        Self {
            id: triple.attribute,
            value: triple.value,
        }
    }
}

impl<S, V> From<(S, V)> for AttributeNode
where
    S: Into<String>,
    V: Into<Value>,
{
    fn from(value: (S, V)) -> Self {
        AttributeNode {
            id: value.0.into(),
            value: value.1.into(),
        }
    }
}
