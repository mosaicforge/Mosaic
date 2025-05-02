use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use grc20_core::{
    entity, error::DatabaseError, indexer_ids, mapping::{Entity, Query}, neo4rs
};

#[derive(Clone, Default, Deserialize, Serialize)]
#[grc20_core::entity]
pub struct Cursor {
    #[grc20(attribute = indexer_ids::CURSOR_ATTRIBUTE)]
    pub cursor: String,
    #[grc20(attribute = indexer_ids::BLOCK_NUMBER_ATTRIBUTE)]
    pub block_number: u64,
    #[grc20(attribute = indexer_ids::BLOCK_TIMESTAMP_ATTRIBUTE)]
    pub block_timestamp: DateTime<Utc>,
    #[grc20(attribute = indexer_ids::VERSION_ATTRIBUTE)]
    pub version_index: String,
}

impl Cursor {
    pub fn new(
        cursor: &str,
        block_number: u64,
        block_timestamp: DateTime<Utc>,
        version: String,
    ) -> Entity<Self> {
        Entity::new(
            indexer_ids::CURSOR_ID,
            Self {
                cursor: cursor.to_string(),
                block_number,
                block_timestamp,
                version_index: version,
            },
        )
        .with_type(indexer_ids::CURSOR_TYPE)
    }

    pub async fn load(neo4j: &neo4rs::Graph) -> Result<Option<Entity<Self>>, DatabaseError> {
        entity::find_one::<Entity<Self>>(
            neo4j,
            indexer_ids::CURSOR_ID,
        )
        .space_id(indexer_ids::INDEXER_SPACE_ID)
        .send()
        .await
    }
}
