pub mod neo4j;
pub mod ipfs_mock;

use grc20_core::block::BlockMetadata;
use chrono::Utc;
use sink::events::{EventHandler, HandlerError};
use grc20_core::neo4rs;
use ipfs::IpfsClient;

pub fn create_handler(neo4j: neo4rs::Graph, ipfs: IpfsClient) -> Result<EventHandler, HandlerError> {
    EventHandler::new_with_ipfs(neo4j, ipfs, None)
}

pub fn create_block_metadata() -> BlockMetadata {
    BlockMetadata {
        cursor: "test-cursor".to_string(),
        block_number: 12345,
        timestamp: Utc::now(),
        request_id: "test-request-id".to_string(),
    }
}
