use chrono::{DateTime, Utc};

#[derive(Clone, Default)]
pub struct BlockMetadata {
    pub cursor: String,
    pub block_number: u64,
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
}
