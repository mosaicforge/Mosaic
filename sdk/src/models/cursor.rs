use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    error::DatabaseError,
    indexer_ids,
    mapping::{self, entity, Entity, Query},
};

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Cursor {
    pub cursor: String,
    pub block_number: u64,
    pub block_timestamp: DateTime<Utc>,
}

impl Cursor {
    pub fn new(cursor: &str, block_number: u64, block_timestamp: DateTime<Utc>) -> Entity<Self> {
        Entity::new(
            indexer_ids::CURSOR_ID,
            Self {
                cursor: cursor.to_string(),
                block_number,
                block_timestamp,
            },
        )
        .with_type(indexer_ids::CURSOR_TYPE)
    }

    pub async fn load(neo4j: &neo4rs::Graph) -> Result<Option<Entity<Self>>, DatabaseError> {
        entity::find_one(
            neo4j,
            indexer_ids::CURSOR_ID,
            indexer_ids::INDEXER_SPACE_ID,
            None,
        )
        .send()
        .await
    }
}

impl mapping::IntoAttributes for Cursor {
    fn into_attributes(self) -> Result<mapping::Attributes, mapping::TriplesConversionError> {
        Ok(mapping::Attributes::default()
            .attribute((indexer_ids::CURSOR_ATTRIBUTE, self.cursor))
            .attribute((indexer_ids::BLOCK_NUMBER_ATTRIBUTE, self.block_number))
            .attribute((indexer_ids::BLOCK_TIMESTAMP_ATTRIBUTE, self.block_timestamp)))
    }
}

impl mapping::FromAttributes for Cursor {
    fn from_attributes(
        mut attributes: mapping::Attributes,
    ) -> Result<Self, mapping::TriplesConversionError> {
        Ok(Self {
            cursor: attributes.pop(indexer_ids::CURSOR_ATTRIBUTE)?,
            block_number: attributes.pop(indexer_ids::BLOCK_NUMBER_ATTRIBUTE)?,
            block_timestamp: attributes.pop(indexer_ids::BLOCK_TIMESTAMP_ATTRIBUTE)?,
        })
    }
}
