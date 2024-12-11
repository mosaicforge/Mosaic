use std::collections::HashMap;

use serde::Deserialize;

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

/// GRC20 Node
#[derive(Debug, Deserialize, PartialEq)]
pub struct Node<T> {
    #[serde(rename = "labels", deserialize_with = "deserialize_labels")]
    pub types: Vec<String>,
    #[serde(flatten)]
    pub attributes: NodeAttributes<T>,
}

impl<T> TryFrom<neo4rs::Node> for Node<T>
where
    T: for<'a> serde::Deserialize<'a>,
{
    type Error = neo4rs::DeError;

    fn try_from(value: neo4rs::Node) -> Result<Self, Self::Error> {
        let labels = value.labels().iter().map(|l| l.to_string()).collect();
        let attributes = value.to()?;
        Ok(Self {
            types: labels,
            attributes,
        })
    }
}

/// Neo4j node representing a GRC20 entity of type `T`.
#[derive(Debug, Deserialize, PartialEq)]
pub struct NodeAttributes<T> {
    pub id: String,
    // pub space_id: String,
    #[serde(flatten)]
    pub attributes: T,
}

fn deserialize_labels<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let labels: neo4rs::Labels = serde::Deserialize::deserialize(deserializer)?;
    Ok(labels.0)
}

impl<T> Node<T> {
    pub fn new(id: String, data: T) -> Self {
        Self {
            types: Vec::new(),
            attributes: NodeAttributes {
                id,
                attributes: data,
            },
        }
    }

    pub fn id(&self) -> &str {
        &self.attributes.id
    }

    pub fn attributes(&self) -> &T {
        &self.attributes.attributes
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
        self.attributes
            .attributes
            .insert(attribute_id, value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use serde::de::IntoDeserializer;

    use super::*;

    #[test]
    pub fn test_deserialize_node() {
        let row = neo4rs::Row::new(
            neo4rs::BoltList {
                value: vec![neo4rs::BoltType::String(neo4rs::BoltString {
                    value: "n".to_string(),
                })],
            },
            neo4rs::BoltList {
                value: vec![neo4rs::BoltType::Node(neo4rs::BoltNode {
                    id: neo4rs::BoltInteger { value: 425 },
                    labels: neo4rs::BoltList {
                        value: vec![neo4rs::BoltType::String(neo4rs::BoltString {
                            value: "9u4zseS3EDXG9ZvwR9RmqU".to_string(),
                        })],
                    },
                    properties: neo4rs::BoltMap {
                        value: HashMap::from([
                            (
                                neo4rs::BoltString {
                                    value: "space_id".to_string(),
                                },
                                neo4rs::BoltType::String(neo4rs::BoltString {
                                    value: "NBDtpHimvrkmVu7vVBXX7b".to_string(),
                                }),
                            ),
                            (
                                neo4rs::BoltString {
                                    value: "GG8Z4cSkjv8CywbkLqVU5M".to_string(),
                                },
                                neo4rs::BoltType::String(neo4rs::BoltString {
                                    value: "Person Posts Page Template".to_string(),
                                }),
                            ),
                            (
                                neo4rs::BoltString {
                                    value: "id".to_string(),
                                },
                                neo4rs::BoltType::String(neo4rs::BoltString {
                                    value: "98wgvodwzidmVA4ryVzGX6".to_string(),
                                }),
                            ),
                        ]),
                    },
                })],
            },
        );
        // let node = neo4rs::BoltType::Node(neo4rs::BoltNode {
        //     id: neo4rs::BoltInteger { value: 425 },
        //     labels: neo4rs::BoltList {
        //         value: vec![neo4rs::BoltType::String(neo4rs::BoltString {
        //             value: "9u4zseS3EDXG9ZvwR9RmqU".to_string(),
        //         })],
        //     },
        //     properties: neo4rs::BoltMap {
        //         value: HashMap::from([
        //             (
        //                 neo4rs::BoltString {
        //                     value: "space_id".to_string(),
        //                 },
        //                 neo4rs::BoltType::String(neo4rs::BoltString {
        //                     value: "NBDtpHimvrkmVu7vVBXX7b".to_string(),
        //                 }),
        //             ),
        //             (
        //                 neo4rs::BoltString {
        //                     value: "GG8Z4cSkjv8CywbkLqVU5M".to_string(),
        //                 },
        //                 neo4rs::BoltType::String(neo4rs::BoltString {
        //                     value: "Person Posts Page Template".to_string(),
        //                 }),
        //             ),
        //             (
        //                 neo4rs::BoltString {
        //                     value: "id".to_string(),
        //                 },
        //                 neo4rs::BoltType::String(neo4rs::BoltString {
        //                     value: "98wgvodwzidmVA4ryVzGX6".to_string(),
        //                 }),
        //             ),
        //         ]),
        //     },
        // });

        // let jd = node.into_deserializer();

        // let result: Node<HashMap<String, serde_json::Value>> = serde_path_to_error::deserialize(jd)
        //     .expect("Failed to deserialize to Node<HashMap<String, serde_json::Value>>");

        #[derive(Deserialize)]
        struct QueryResult {
            n: Node<HashMap<String, serde_json::Value>>,
        }

        let result = row.get::<Node<HashMap<String, serde_json::Value>>>("n").expect("Failed to deserialize to QueryResult");

    }

    #[test]
    pub fn test_node_conversion() {
        let node = neo4rs::Node::new(neo4rs::BoltNode {
            id: neo4rs::BoltInteger { value: 425 },
            labels: neo4rs::BoltList {
                value: vec![neo4rs::BoltType::String(neo4rs::BoltString {
                    value: "9u4zseS3EDXG9ZvwR9RmqU".to_string(),
                })],
            },
            properties: neo4rs::BoltMap {
                value: HashMap::from([
                    (
                        neo4rs::BoltString {
                            value: "space_id".to_string(),
                        },
                        neo4rs::BoltType::String(neo4rs::BoltString {
                            value: "NBDtpHimvrkmVu7vVBXX7b".to_string(),
                        }),
                    ),
                    (
                        neo4rs::BoltString {
                            value: "GG8Z4cSkjv8CywbkLqVU5M".to_string(),
                        },
                        neo4rs::BoltType::String(neo4rs::BoltString {
                            value: "Person Posts Page Template".to_string(),
                        }),
                    ),
                    (
                        neo4rs::BoltString {
                            value: "id".to_string(),
                        },
                        neo4rs::BoltType::String(neo4rs::BoltString {
                            value: "98wgvodwzidmVA4ryVzGX6".to_string(),
                        }),
                    ),
                ]),
            },
        });

        let node: Node<HashMap<String, serde_json::Value>> = node.try_into()
            .expect("Failed to convert neo4rs::Node to Node<HashMap<String, neo4rs::BoltType>>");
    
        assert_eq!(
            node,
            Node {
                types: vec!["9u4zseS3EDXG9ZvwR9RmqU".to_string()],
                attributes: NodeAttributes {
                    id: "98wgvodwzidmVA4ryVzGX6".to_string(),
                    attributes: HashMap::from([
                        (
                            "space_id".to_string(),
                            serde_json::Value::String("NBDtpHimvrkmVu7vVBXX7b".to_string())
                        ),
                        (
                            "GG8Z4cSkjv8CywbkLqVU5M".to_string(),
                            serde_json::Value::String("Person Posts Page Template".to_string())
                        ),
                    ])
                }
            }
        )
    }
}
