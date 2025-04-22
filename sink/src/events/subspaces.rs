use grc20_core::{block::BlockMetadata, indexer_ids, mapping::query_utils::Query, network_ids, pb::geo};
use grc20_sdk::models::{space, space::ParentSpace};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_subspace_added(
        &self,
        subspace_added: &geo::SubspaceAdded,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        tracing::info!(
            "Block #{} ({}): Creating subspace relation with plugin_address = {}",
            block.block_number,
            block.timestamp,
            checksum_address(&subspace_added.plugin_address)
        );
        let subspace_id = &space::new_id(network_ids::GEO, &subspace_added.subspace); 
        let parent_space_id = &space::new_id(network_ids::GEO, &subspace_added.dao_address);

        ParentSpace::new(&subspace_id, &parent_space_id)
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await?;

        Ok(())
    }

    pub async fn handle_subspace_removed(
        &self,
        subspace_removed: &geo::SubspaceRemoved,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let subspace_id = space::new_id(network_ids::GEO, &subspace_removed.subspace);
        let parent_space_id = space::new_id(network_ids::GEO, &subspace_removed.dao_address);

        ParentSpace::remove(&self.neo4j, block, &subspace_id, &parent_space_id).await?;

        tracing::info!(
            "Block #{} ({}): Removed subspace {} from space {}",
            block.block_number,
            block.timestamp,
            subspace_removed.subspace,
            parent_space_id,
        );

        Ok(())
    }
}
