use futures::join;
use grc20_core::{block::BlockMetadata, indexer_ids, mapping::query_utils::Query, pb::geo};
use grc20_sdk::models::{space, space::ParentSpace};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_subspace_added(
        &self,
        subspace_added: &geo::SubspaceAdded,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        match join!(
            space::find_entity_by_space_plugin_address(&self.neo4j, &subspace_added.plugin_address),
            space::find_entity_by_dao_address(&self.neo4j, &subspace_added.subspace)
        ) {
            (Ok(Some(parent_space)), Ok(Some(subspace))) => {
                tracing::info!(
                    "Block #{} ({}): Creating subspace relation with plugin_address = {}",
                    block.block_number,
                    block.timestamp,
                    checksum_address(&subspace_added.plugin_address)
                );
                ParentSpace::new(&subspace.id, &parent_space.id)
                    .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
                    .send()
                    .await?;
            }
            (Ok(None), Ok(_)) => {
                tracing::warn!(
                    "Block #{} ({}): Could not create subspace relation: parent space with plugin_address = {} not found",
                    block.block_number,
                    block.timestamp,
                    checksum_address(&subspace_added.plugin_address)
                );
            }
            (Ok(Some(_)), Ok(None)) => {
                tracing::warn!(
                    "Block #{} ({}): Could not create subspace relation: subspace with dao_address = {} not found",
                    block.block_number,
                    block.timestamp,
                    checksum_address(&subspace_added.subspace)
                );
            }
            (Err(e), _) | (_, Err(e)) => {
                Err(HandlerError::from(e))?;
            }
        };

        Ok(())
    }
}
