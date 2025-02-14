use std::{
    collections::{hash_map, HashMap},
    fmt::Display, iter::Map,
};
use futures::TryStreamExt;

use neo4rs::BoltType;
use serde::{ser::SerializeMap, Deserialize, Serialize};

use crate::pb;

/// Data is written/read to/from the database in the following format:
/// T <-> Triples <-> Neo4j

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Triples(pub(crate) HashMap<String, Triple>);

impl Triples {
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            items: self.0.iter(),
        }
    }
}

impl From<Vec<Triple>> for Triples {
    fn from(triples: Vec<Triple>) -> Self {
        Triples(triples.into_iter().map(|triple| (triple.attribute.clone(), triple)).collect())
    }
}

impl IntoIterator for Triples {
    type Item = (String, Triple);
    type IntoIter = std::collections::hash_map::IntoIter<String, Triple>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct TripleBoltRow {
    #[serde(rename = "n")]
    entity: neo4rs::Node,
    #[serde(rename = "r")]
    rel: neo4rs::Relation,
    #[serde(rename = "m")]
    value: neo4rs::Node,
}

impl Extend<TripleBoltRow> for Triples {
    fn extend<T: IntoIterator<Item = TripleBoltRow>>(&mut self, iter: T) {
        for row in iter {
            match row.value.to::<Triple>() {
                Ok(triple) => {
                    self.0.insert(triple.attribute.clone(), triple);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse Triple: {e:?} {:?}", row.value);
                }
            }
        }
    }
}

impl Into<BoltType> for Triples {
    fn into(self) -> BoltType {
        BoltType::List(neo4rs::BoltList { 
            value: self.into_iter().map(|(key, value)| {
                let mut bolt_map = HashMap::new();
                bolt_map.insert(neo4rs::BoltString { value: "attribute".into() }, key.into());
                bolt_map.insert(neo4rs::BoltString { value: "value".into() }, value.value.into());
                bolt_map.insert(neo4rs::BoltString { value: "value_type".into() }, value.value_type.to_string().into());
                if let Some(format) = value.options.format {
                    bolt_map.insert(neo4rs::BoltString { value: "format".into() }, format.into());
                }
                if let Some(unit) = value.options.unit {
                    bolt_map.insert(neo4rs::BoltString { value: "unit".into() }, unit.into());
                }
                if let Some(language) = value.options.language {
                    bolt_map.insert(neo4rs::BoltString { value: "language".into() }, language.into());
                }


                BoltType::Map(neo4rs::BoltMap {value: bolt_map})
            }).collect()
        })
    }
}

/// Trait to convert a type into Triples
pub trait IntoTriples {
    fn into_triples(self) -> Triples;
}

impl IntoTriples for HashMap<String, Triple> {
    fn into_triples(self) -> Triples {
        Triples(self)
    }
}

impl IntoTriples for () {
    fn into_triples(self) -> Triples {
        Triples::default()
    }
}


#[derive(Debug, thiserror::Error)]
pub enum TriplesError {
    #[error("Invalid value: must serialize to serde_json::Map of (String, Scalar) values")]
    InvalidValue,
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Failible trait to convert a type into Triples
pub trait TryIntoTriples {
   fn try_into_triples(self) -> Result<Triples, TriplesError>;
}

impl TryIntoTriples for Triples {
    fn try_into_triples(self) -> Result<Triples, TriplesError> {
        Ok(self)
    }
}

impl<T: Serialize> TryIntoTriples for T {
    fn try_into_triples(self) -> Result<Triples, TriplesError> {
        if let serde_json::Value::Object(map) = serde_json::to_value(&self)? {
            Ok(Triples(
                map.into_iter()
                    .map(|(key, value)| {
                        match value {
                            serde_json::Value::String(value) => Ok((
                                key.clone(),
                                Triple {
                                    attribute: key,
                                    value,
                                    value_type: ValueType::Text,
                                    options: Options::default(),
                                },
                            )),
                            serde_json::Value::Number(number) if number.is_i64() => Ok((
                                key.clone(),
                                Triple {
                                    attribute: key,
                                    value: number.as_i64().unwrap().to_string(),
                                    value_type: ValueType::Number,
                                    options: Options::default(),
                                },
                            )),
                            serde_json::Value::Number(number) if number.is_f64() => Ok((
                                key.clone(),
                                Triple {
                                    attribute: key,
                                    value: number.as_f64().unwrap().to_string(),
                                    value_type: ValueType::Number,
                                    options: Options::default(),
                                },
                            )),
                            serde_json::Value::Number(number) if number.is_u64() => Ok((
                                key.clone(),
                                Triple {
                                    attribute: key,
                                    value: number.as_u64().unwrap().to_string(),
                                    value_type: ValueType::Number,
                                    options: Options::default(),
                                },
                            )),
                            serde_json::Value::Bool(value) => Ok((
                                key.clone(),
                                Triple {
                                    attribute: key,
                                    value: value.to_string(),
                                    value_type: ValueType::Checkbox,
                                    options: Options::default(),
                                },
                            )),
                            _ => Err(TriplesError::InvalidValue),
                        }
                    })
                    .collect::<Result<HashMap<_, _>, _>>()?,
            ))
        } else {
            Err(TriplesError::InvalidValue)
        }
    }    
}

/// Trait to convert Triples into a type
pub trait FromTriples {
    fn from_triples(triples: Triples) -> Self;
}

impl FromTriples for HashMap<String, Triple> {
    fn from_triples(triples: Triples) -> Self {
        triples.0
    }
}

impl FromTriples for Triples {
    fn from_triples(triples: Triples) -> Self {
        triples
    }
}

impl FromTriples for () {
    fn from_triples(_: Triples) -> Self {
        ()
    }
}

/// Failible trait to convert Triples into a type
pub trait TryFromTriples: Sized {
   fn try_from_triples(triples: Triples) -> Result<Self, TriplesError>;
}

impl TryFromTriples for Triples {
    fn try_from_triples(triples: Triples) -> Result<Self, TriplesError> {
        Ok(triples)
    }
}


impl<T: for<'a> Deserialize<'a>> TryFromTriples for T {
    fn try_from_triples(triples: Triples) -> Result<Self, TriplesError> {
        // Triples -> serde_json::Value -> T
        let map = triples.0.into_iter()
            .map(|(key, value)| {
                match value.value_type {
                    ValueType::Text => Ok((key, serde_json::Value::String(value.value))),
                    ValueType::Number => Ok(value.value.parse().map(|value| (key, value))?),
                    ValueType::Checkbox => Ok(value.value.parse().map(|value| (key, value))?),
                    _ => Err(TriplesError::InvalidValue),
                }
            })
            .collect::<Result<serde_json::Value, _>>()?;

        Ok(serde_json::from_value(map)?)
    }
}

pub struct Iter<'a> {
    items: hash_map::Iter<'a, String, Triple>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a String, &'a Triple);

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Triple {
    pub attribute: String,
    pub value: String,
    pub value_type: ValueType,

    #[serde(flatten)]
    pub options: Options,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Options {
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValueType {
    #[default]
    Text,
    Number,
    Checkbox,
    Url,
    Time,
    Point,
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Text => write!(f, "TEXT"),
            ValueType::Number => write!(f, "NUMBER"),
            ValueType::Checkbox => write!(f, "CHECKBOX"),
            ValueType::Url => write!(f, "URL"),
            ValueType::Time => write!(f, "TIME"),
            ValueType::Point => write!(f, "POINT"),
        }
    }
}

impl TryFrom<pb::ipfs::ValueType> for ValueType {
    type Error = String;

    fn try_from(value: pb::ipfs::ValueType) -> Result<Self, Self::Error> {
        match value {
            pb::ipfs::ValueType::Text => Ok(ValueType::Text),
            pb::ipfs::ValueType::Number => Ok(ValueType::Number),
            pb::ipfs::ValueType::Checkbox => Ok(ValueType::Checkbox),
            pb::ipfs::ValueType::Url => Ok(ValueType::Url),
            pb::ipfs::ValueType::Time => Ok(ValueType::Time),
            pb::ipfs::ValueType::Point => Ok(ValueType::Point),
            pb::ipfs::ValueType::Unknown => Err("Unknown ValueType".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use testcontainers::{
        core::{IntoContainerPort, WaitFor},
        runners::AsyncRunner,
        GenericImage, ImageExt,
    };

    const BOLT_PORT: u16 = 7687;
    const HTTP_PORT: u16 = 7474;

    #[test]
    fn test_triples_conversion() {
        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        struct Foo {
            foo: String,
            bar: i64,
        }

        let foo = Foo {
            foo: "Hello, World!".to_string(),
            bar: 123,
        };

        let triples = Triples(HashMap::from([
            (
                "foo".to_string(),
                Triple {
                    attribute: "foo".to_string(),
                    value: "Hello, World!".to_string(),
                    value_type: ValueType::Text,
                    options: Options::default(),
                },
            ),
            (
                "bar".to_string(),
                Triple {
                    attribute: "bar".to_string(),
                    value: "123".to_string(),
                    value_type: ValueType::Number,
                    options: Options::default(),
                },
            ),
        ]));

        assert_eq!(
            foo.clone().try_into_triples().expect("Failed to convert Foo into Triples"),
            triples
        );

        let foo2 = Foo::try_from_triples(triples).expect("Failed to convert Triples into Foo");

        assert_eq!(foo2, foo);
    }

    #[tokio::test]
    async fn test_triples_insert() {
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
    
        let triples = Triples(HashMap::from([
            (
                "foo".to_string(),
                Triple {
                    attribute: "foo".to_string(),
                    value: "Hello, World!".to_string(),
                    value_type: ValueType::Text,
                    options: Options::default(),
                },
            ),
            (
                "bar".to_string(),
                Triple {
                    attribute: "bar".to_string(),
                    value: "123".to_string(),
                    value_type: ValueType::Number,
                    options: Options::default(),
                },
            ),
        ]));

        let query = r#"
        MERGE (n)
        WITH n
        UNWIND $triples AS triple
        MERGE (n) -[r:$(triple["attribute"]) {version_index: 0, space_id: "abc"}]-> (m)
        SET m += triple
        RETURN n, r, m
        "#;

        let result = neo4j
            .execute(neo4rs::query(query).param("triples", triples.clone()))
            .await
            .expect("Failed to insert triples")
            .into_stream_as::<TripleBoltRow>()
            .try_collect::<Triples>()
            .await
            .expect("Failed to convert triples");

        assert_eq!(triples, result);
    }

    #[tokio::test]
    async fn test_struct_insert() {
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
    
        #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
        struct Foo {
            foo: String,
            bar: i64,
        }

        let foo = Foo {
            foo: "Hello, World!".to_string(),
            bar: 123,
        };

        let query = r#"
        MERGE (n)
        WITH n
        UNWIND $triples AS triple
        MERGE (n) -[r:TRIPLE {revision: 0, attribute: triple["attribute"]}]-> (m)
        SET m += triple
        RETURN n, r, m
        "#;

        let result = Foo::try_from_triples(neo4j
            .execute(neo4rs::query(query).param("triples", foo.clone().try_into_triples().expect("Failed to convert Foo into Triples")))
            .await
            .expect("Failed to insert triples")
            .into_stream_as::<TripleBoltRow>()
            .try_collect::<Triples>()
            .await
            .expect("Failed to convert triples")).expect("Failed to convert Triples into Foo");

        assert_eq!(foo, result);
    }
}
