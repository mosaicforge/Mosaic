use std::{
    collections::{hash_map, HashMap},
    fmt::Display, iter::Map,
};

use neo4rs::BoltType;
use serde::{ser::SerializeMap, Deserialize, Serialize};

use crate::pb;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Triples(pub(crate) HashMap<String, Triple>);

impl IntoIterator for Triples {
    type Item = (String, Triple);
    type IntoIter = std::collections::hash_map::IntoIter<String, Triple>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub trait IntoTriples {
    fn into_triples(self) -> Triples;
}

pub trait TryIntoTriples {
    type Error;

    fn try_into_triples(self) -> Result<Triples, Self::Error>;
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

impl<T: Serialize> TryIntoTriples for T {
    type Error = TriplesError;

    fn try_into_triples(self) -> Result<Triples, Self::Error> {
        if let serde_json::Value::Object(map) = serde_json::to_value(&self)? {
            Ok(Triples(
                map.into_iter()
                    .map(|(key, value)| {
                        match value {
                            serde_json::Value::String(value) => Ok((
                                key,
                                Triple {
                                    value,
                                    value_type: ValueType::Text,
                                    options: Options::default(),
                                },
                            )),
                            serde_json::Value::Number(number) if number.is_i64() => Ok((
                                key,
                                Triple {
                                    value: number.as_i64().unwrap().to_string(),
                                    value_type: ValueType::Number,
                                    options: Options::default(),
                                },
                            )),
                            serde_json::Value::Number(number) if number.is_f64() => Ok((
                                key,
                                Triple {
                                    value: number.as_f64().unwrap().to_string(),
                                    value_type: ValueType::Number,
                                    options: Options::default(),
                                },
                            )),
                            serde_json::Value::Number(number) if number.is_u64() => Ok((
                                key,
                                Triple {
                                    value: number.as_u64().unwrap().to_string(),
                                    value_type: ValueType::Number,
                                    options: Options::default(),
                                },
                            )),
                            serde_json::Value::Bool(value) => Ok((
                                key,
                                Triple {
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

pub trait FromTriples {
    fn from_triples(triples: Triples) -> Self;
}

pub trait TryFromTriples: Sized {
    type Error;

    fn try_from_triples(triples: Triples) -> Result<Self, Self::Error>;
}

impl FromTriples for HashMap<String, Triple> {
    fn from_triples(triples: Triples) -> Self {
        triples.0
    }
}

impl FromTriples for () {
    fn from_triples(_: Triples) -> Self {
        ()
    }
}

impl<T: for<'a> Deserialize<'a>> TryFromTriples for T {
    type Error = TriplesError;

    fn try_from_triples(triples: Triples) -> Result<Self, Self::Error> {
        let map = triples.0.into_iter().map(|(key, value)| {
            match value.value_type {
                ValueType::Text => Ok((key, serde_json::Value::String(value.value))),
                ValueType::Number => Ok(value.value.parse().map(|value| (key, value))?),
                ValueType::Checkbox => Ok(value.value.parse().map(|value| (key, value))?),
                _ => Err(TriplesError::InvalidValue),
            }
        }).collect::<Map<_, _>>();

        Ok(serde_json::from_value(serde_json::Value::Object(map))?)
    }
}

impl Triples {
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            items: self.0.iter(),
        }
    }

    pub(crate) fn to_bolt_type(self) -> BoltType {
        BoltType::List(neo4rs::BoltList { 
            value: self.into_iter().map(|(key, value)| {
                let mut bolt_map = HashMap::new();
                bolt_map.insert(neo4rs::BoltString { value: "attribute".into() }, key.into());
                bolt_map.insert(neo4rs::BoltString { value: "value".into() }, value.value.into());
                bolt_map.insert(neo4rs::BoltString { value: "type".into() }, value.value_type.to_string().into());
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

pub struct Iter<'a> {
    items: hash_map::Iter<'a, String, Triple>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a String, &'a Triple);

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
    }
}

// impl Serialize for Triples {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut map = serializer.serialize_map(None)?;
//         for (key, value) in &self.0 {
//             map.serialize_entry(key, &value.value)?;
//             map.serialize_entry(&format!("{}.type", key), &value.value_type)?;
//             if let Some(ref format) = value.options.format {
//                 map.serialize_entry(&format!("{}.options.format", key), format)?;
//             }
//             if let Some(ref unit) = value.options.unit {
//                 map.serialize_entry(&format!("{}.options.unit", key), unit)?;
//             }
//             if let Some(ref language) = value.options.language {
//                 map.serialize_entry(&format!("{}.options.language", key), language)?;
//             }
//         }
//         map.end()
//     }
// }

// impl<'de> Deserialize<'de> for Triples {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         struct TriplesVisitor;

//         impl<'de> serde::de::Visitor<'de> for TriplesVisitor {
//             type Value = Triples;

//             fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//                 formatter.write_str("a map representing triples")
//             }

//             fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
//             where
//                 M: serde::de::MapAccess<'de>,
//             {
//                 let mut triples = HashMap::new();

//                 while let Some(key) = map.next_key::<String>()? {
//                     match key.split('.').collect::<Vec<_>>()[..] {
//                         [key] => {
//                             let value = map.next_value::<String>()?;
//                             triples
//                                 .entry(key.to_string())
//                                 .or_insert(Triple::default())
//                                 .value = value;
//                         }
//                         [key, "type"] => {
//                             let value = map.next_value::<ValueType>()?;
//                             triples
//                                 .entry(key.to_string())
//                                 .or_insert(Triple::default())
//                                 .value_type = value;
//                         }
//                         [key, "options", "format"] => {
//                             let value = map.next_value::<String>()?;
//                             triples
//                                 .entry(key.to_string())
//                                 .or_insert(Triple::default())
//                                 .options
//                                 .format = Some(value);
//                         }
//                         [key, "options", "unit"] => {
//                             let value = map.next_value::<String>()?;
//                             triples
//                                 .entry(key.to_string())
//                                 .or_insert(Triple::default())
//                                 .options
//                                 .unit = Some(value);
//                         }
//                         [key, "options", "language"] => {
//                             let value = map.next_value::<String>()?;
//                             triples
//                                 .entry(key.to_string())
//                                 .or_insert(Triple::default())
//                                 .options
//                                 .language = Some(value);
//                         }
//                         _ => return Err(serde::de::Error::custom(format!("Invalid key: {}", key))),
//                     }
//                 }

//                 Ok(Triples(triples))
//             }
//         }

//         deserializer.deserialize_map(TriplesVisitor)
//     }
// }

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Triple {
    pub value: String,
    pub value_type: ValueType,

    #[serde(flatten)]
    pub options: Options,
}

// impl Serialize for Triple {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut map = serializer.serialize_map(None)?;
//         map.serialize_entry("", &self.value)?;
//         map.serialize_entry(".type", &self.value_type)?;
//         if let Some(ref format) = self.options.format {
//             map.serialize_entry(".options.format", format)?;
//         }
//         if let Some(ref unit) = self.options.unit {
//             map.serialize_entry(".options.unit", unit)?;
//         }
//         if let Some(ref language) = self.options.language {
//             map.serialize_entry(".options.language", language)?;
//         }
//         map.end()
//     }
// }

// impl<'de> Deserialize<'de> for Triple {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         #[derive(Deserialize)]
//         struct TripleHelper {
//             #[serde(rename = "")]
//             value: String,
//             #[serde(rename = ".type")]
//             r#type: ValueType,
//             #[serde(rename = ".options.format")]
//             format: Option<String>,
//             #[serde(rename = ".options.unit")]
//             unit: Option<String>,
//             #[serde(rename = ".options.language")]
//             language: Option<String>,
//         }

//         let helper = TripleHelper::deserialize(deserializer)?;
//         Ok(Triple {
//             value: helper.value,
//             value_type: helper.r#type,
//             options: Options {
//                 format: helper.format,
//                 unit: helper.unit,
//                 language: helper.language,
//             },
//         })
//     }
// }

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Options {
    pub format: Option<String>,
    pub unit: Option<String>,
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

    #[test]
    fn test_triples_conversion() {
        #[derive(Debug, Serialize, PartialEq)]
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
                    value: "Hello, World!".to_string(),
                    value_type: ValueType::Text,
                    options: Options::default(),
                },
            ),
            (
                "bar".to_string(),
                Triple {
                    value: "123".to_string(),
                    value_type: ValueType::Number,
                    options: Options::default(),
                },
            ),
        ]));

        assert_eq!(
            foo.try_into_triples().expect("Failed to convert Foo into Triples"),
            triples
        );

        let foo2 = Foo::try_from_triples(triples).expect("Failed to convert Triples into Foo");

        assert_eq!(foo2, foo);
    }
}
