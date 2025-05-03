pub mod ipfs_mock;
pub mod neo4j;

use grc20_core::block::BlockMetadata;
use grc20_core::neo4rs;
use ipfs::IpfsClient;
use sink::events::{EventHandler, HandlerError};

pub fn create_handler(
    neo4j: neo4rs::Graph,
    ipfs: IpfsClient,
) -> Result<EventHandler, HandlerError> {
    EventHandler::new_with_ipfs(neo4j, ipfs, None)
}

pub fn create_block_metadata() -> BlockMetadata {
    BlockMetadata::default()
}
