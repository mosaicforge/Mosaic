use std::collections::HashMap;

use neo4rs::BoltType;

use super::{TriplesConversionError, Value};

#[derive(Clone, Debug)]
pub enum AggregationDirection {
    Up,
    Down,
    Bidirectional,
}

impl From<AggregationDirection> for Value {
    fn from(direction: AggregationDirection) -> Self {
        match direction {
            AggregationDirection::Up => Value::text("Up"),
            AggregationDirection::Down => Value::text("Down"),
            AggregationDirection::Bidirectional => Value::text("Bidirectional"),
        }
    }
}

impl TryFrom<Value> for AggregationDirection {
    type Error = TriplesConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.value.as_str() {
            "Up" => Ok(AggregationDirection::Up),
            "Down" => Ok(AggregationDirection::Down),
            "Bidirectional" => Ok(AggregationDirection::Bidirectional),
            _ => Err(TriplesConversionError::InvalidValue(format!(
                "Invalid aggregation direction: {}",
                value.value
            ))),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct SpaceRanking {
    pub space_id: String,
    pub depth: usize,
}

impl From<SpaceRanking> for BoltType {
    fn from(value: SpaceRanking) -> Self {
        let mut map = HashMap::new();
        map.insert(neo4rs::BoltString { value: "space_id".into() }, value.space_id.into());
        map.insert(neo4rs::BoltString { value: "depth".into() }, (value.depth as i64).into());
        BoltType::Map(neo4rs::BoltMap {
            value: map,
        })
    }
}