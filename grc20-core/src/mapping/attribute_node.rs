use std::collections::HashMap;

use chrono::{DateTime, Utc};
use neo4rs::BoltType;
use serde::Deserialize;

use super::{Triple, TriplesConversionError, Value};

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

impl From<AttributeNode> for BoltType {
    fn from(attr: AttributeNode) -> Self {
        let mut map = HashMap::new();
        map.insert(neo4rs::BoltString { value: "id".into() }, attr.id.into());
        map.insert(
            neo4rs::BoltString {
                value: "value".into(),
            },
            attr.value.value.into(),
        );
        map.insert(
            neo4rs::BoltString {
                value: "value_type".into(),
            },
            attr.value.value_type.to_string().into(),
        );
        if let Some(format) = attr.value.options.format {
            map.insert(
                neo4rs::BoltString {
                    value: "format".into(),
                },
                format.into(),
            );
        }
        if let Some(unit) = attr.value.options.unit {
            map.insert(
                neo4rs::BoltString {
                    value: "unit".into(),
                },
                unit.into(),
            );
        }
        if let Some(language) = attr.value.options.language {
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

impl TryFrom<AttributeNode> for String {
    type Error = TriplesConversionError;

    fn try_from(attr: AttributeNode) -> Result<Self, Self::Error> {
        attr.value.try_into()
    }
}

impl TryFrom<AttributeNode> for i64 {
    type Error = TriplesConversionError;

    fn try_from(attr: AttributeNode) -> Result<Self, Self::Error> {
        attr.value.try_into()
    }
}

impl TryFrom<AttributeNode> for u64 {
    type Error = TriplesConversionError;

    fn try_from(attr: AttributeNode) -> Result<Self, Self::Error> {
        attr.value.try_into()
    }
}

impl TryFrom<AttributeNode> for f64 {
    type Error = TriplesConversionError;

    fn try_from(attr: AttributeNode) -> Result<Self, Self::Error> {
        attr.value.try_into()
    }
}

impl TryFrom<AttributeNode> for bool {
    type Error = TriplesConversionError;

    fn try_from(attr: AttributeNode) -> Result<Self, Self::Error> {
        attr.value.try_into()
    }
}

impl TryFrom<AttributeNode> for DateTime<Utc> {
    type Error = TriplesConversionError;

    fn try_from(attr: AttributeNode) -> Result<Self, Self::Error> {
        attr.value.try_into()
    }
}
