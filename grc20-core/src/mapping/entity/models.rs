use neo4rs::BoltType;
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::mapping::value::models::{Value, ValueConversionError};
use crate::pb;

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),
    #[error("Value conversion error: {0}")]
    ValueConversionError(#[from] ValueConversionError),
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Entity {
    pub id: Uuid,
    pub values: HashMap<Uuid, Vec<Value>>,
}

impl Entity {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            values: HashMap::new(),
        }
    }

    pub fn value(mut self, space_id: impl Into<Uuid>, value: impl Into<Value>) -> Self {
        self.values
            .entry(space_id.into())
            .or_default()
            .push(value.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateEntity {
    pub id: Uuid,
    pub values: Vec<Value>,
    pub embedding: Option<Vec<f32>>,
}

impl UpdateEntity {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            values: Vec::new(),
            embedding: None,
        }
    }

    pub fn value(mut self, value: impl Into<Value>) -> Self {
        self.values.push(value.into());
        self
    }

    pub fn values(mut self, values: Vec<Value>) -> Self {
        self.values = values;
        self
    }

    pub fn embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
}

impl TryFrom<pb::grc20::Entity> for UpdateEntity {
    type Error = ConversionError;

    fn try_from(entity: pb::grc20::Entity) -> Result<Self, Self::Error> {
        let id = Uuid::from_slice(&entity.id).map_err(|_| {
            ConversionError::InvalidUuid(format!("Invalid entity ID: {:?}", entity.id))
        })?;

        let values: Result<Vec<Value>, ValueConversionError> =
            entity.values.into_iter().map(Value::try_from).collect();

        Ok(Self {
            id,
            values: values?,
            embedding: None,
        })
    }
}

impl From<UpdateEntity> for BoltType {
    fn from(entity: UpdateEntity) -> Self {
        let mut map = HashMap::new();

        map.insert(
            neo4rs::BoltString { value: "id".into() },
            entity.id.to_string().into(),
        );

        let values: Vec<BoltType> = entity.values.into_iter().map(BoltType::from).collect();

        map.insert(
            neo4rs::BoltString {
                value: "values".into(),
            },
            values.into(),
        );

        map.insert(
            neo4rs::BoltString {
                value: "embedding".into(),
            },
            entity.embedding.into(),
        );

        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnsetEntityValues {
    pub id: Uuid,
    pub properties: Vec<Uuid>,
}

impl TryFrom<pb::grc20::UnsetEntityValues> for UnsetEntityValues {
    type Error = ConversionError;

    fn try_from(unset: pb::grc20::UnsetEntityValues) -> Result<Self, Self::Error> {
        let id = Uuid::from_slice(&unset.id).map_err(|_| {
            ConversionError::InvalidUuid(format!("Invalid entity ID: {:?}", unset.id))
        })?;

        let properties: Result<Vec<Uuid>, ConversionError> = unset
            .properties
            .into_iter()
            .map(|prop| {
                Uuid::from_slice(&prop).map_err(|_| {
                    ConversionError::InvalidUuid(format!("Invalid property ID: {:?}", prop))
                })
            })
            .collect();

        Ok(Self {
            id,
            properties: properties?,
        })
    }
}

impl From<UnsetEntityValues> for BoltType {
    fn from(unset: UnsetEntityValues) -> Self {
        let mut map = HashMap::new();

        map.insert(
            neo4rs::BoltString { value: "id".into() },
            unset.id.to_string().into(),
        );

        let mut properties_map = HashMap::new();
        for property_id in unset.properties {
            properties_map.insert(
                neo4rs::BoltString {
                    value: property_id.to_string(),
                },
                BoltType::Null(neo4rs::BoltNull),
            );
        }

        map.insert(
            neo4rs::BoltString {
                value: "properties".into(),
            },
            BoltType::Map(neo4rs::BoltMap {
                value: properties_map,
            }),
        );

        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}

impl From<Entity> for BoltType {
    fn from(entity: Entity) -> Self {
        let mut map = HashMap::new();
        map.insert(
            neo4rs::BoltString { value: "id".into() },
            entity.id.to_string().into(),
        );

        // Build the "spaces" array
        let spaces: Vec<BoltType> = entity
            .values
            .into_iter()
            .map(|(space_id, values)| {
                let mut space_map = HashMap::new();
                space_map.insert(
                    neo4rs::BoltString {
                        value: "space_id".into(),
                    },
                    space_id.to_string().into(),
                );
                // Convert Vec<Value> to Vec<BoltType>
                let values: Vec<BoltType> = values.into_iter().map(BoltType::from).collect();
                space_map.insert(
                    neo4rs::BoltString {
                        value: "values".into(),
                    },
                    values.into(),
                );
                BoltType::Map(neo4rs::BoltMap { value: space_map })
            })
            .collect();

        map.insert(
            neo4rs::BoltString {
                value: "spaces".into(),
            },
            spaces.into(),
        );

        BoltType::Map(neo4rs::BoltMap { value: map })
    }
}
