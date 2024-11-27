use kg_core::{models, pb::geo, system_ids};
use web3_utils::checksum_address;
use crate::kg::mapping::Node;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_subspace_added(
        &self,
        subspace_added: &geo::SubspaceAdded,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space = self
            .kg
            .get_space_by_space_plugin_address(&subspace_added.plugin_address)
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

        if let Some(space) = space {
            let subspace = models::Subspace {
                id: checksum_address(&subspace_added.subspace, None),
                parent_space: space.id.clone(),
            };

            self.kg
                .upsert_node(
                    system_ids::INDEXER_SPACE_ID,
                    block,
                    Node::new(subspace.id.clone(), subspace.clone())
                        .with_type(system_ids::INDEXED_SPACE),
                )
                .await
                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

            tracing::info!(
                "Block #{} ({}): Subspace {} added to space {}",
                block.block_number,
                block.timestamp,
                subspace.id,
                space.id
            );
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not create subspace for unknown space with plugin address = {}",
                block.block_number,
                block.timestamp,
                subspace_added.plugin_address
            );
        }

        Ok(())
    }
}
