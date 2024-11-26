use chrono::{DateTime, Utc};
use ipfs::IpfsClient;
use kg_core::ids::create_geo_id;
use substreams_sink_rust::pb::sf::substreams::rpc::v2::BlockScopedData;

use crate::kg;

pub struct BlockMetadata {
    pub(crate) cursor: String,
    pub(crate) block_number: u64,
    pub(crate) timestamp: DateTime<Utc>,
    pub(crate) request_id: String,
}

impl BlockMetadata {
    pub fn from_substreams_block(block: &BlockScopedData) -> Self {
        let clock = block.clock.as_ref().unwrap();
        let timestamp = DateTime::from_timestamp(
            clock.timestamp.as_ref().unwrap().seconds,
            clock.timestamp.as_ref().unwrap().nanos as u32,
        )
        .expect("received timestamp should always be valid");

        Self {
            cursor: block.cursor.clone(),
            block_number: clock.number,
            timestamp,
            request_id: create_geo_id(),
        }
    }
}


#[derive(thiserror::Error, Debug)]
pub enum HandlerError {
    #[error("IPFS error: {0}")]
    IpfsError(#[from] ipfs::Error),

    #[error("prost error: {0}")]
    Prost(#[from] prost::DecodeError),
    
    // #[error("KG error: {0}")]
    // KgError(#[from] kg::Error),

    #[error("Error processing event: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub type Result<T> = std::result::Result<T, HandlerError>;

pub struct EventHandler {
    pub(crate) ipfs: IpfsClient,
    pub(crate) kg: kg::Client,
}

impl EventHandler {
    pub fn new(kg: kg::Client) -> Self {
        Self {
            ipfs: IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/"),
            kg,
        }
    }
}