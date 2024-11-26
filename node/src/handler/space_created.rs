use std::collections::HashMap;

use futures::{stream, StreamExt, TryStreamExt};
use kg_core::{ids::create_space_id, models::{Space, SpaceType}, network_ids, pb::{geo, grc20}};
use crate::web3_utils::checksum_address;

use super::{handler::{BlockMetadata, HandlerError}, EventHandler};

impl EventHandler {
    /// Handles `GeoSpaceCreated` events. `ProposalProcessed` events are used to determine
    /// the space's ID in cases where the space is imported.
    ///
    /// The method returns the IDs of the spaces that were successfully created.
    pub async fn handle_spaces_created(
        &self,
        spaces_created: &[geo::GeoSpaceCreated],
        proposals_processed: &[geo::ProposalProcessed],
        block: &BlockMetadata,
    ) -> Result<Vec<String>, HandlerError> {
        // Match the space creation events with their corresponding initial proposal (if any)
        let initial_proposals = spaces_created
            .iter()
            .filter_map(|event| {
                proposals_processed
                    .iter()
                    .find(|proposal| {
                        checksum_address(&proposal.plugin_address, None)
                            == checksum_address(&event.space_address, None)
                    })
                    .map(|proposal| (event.space_address.clone(), proposal))
            })
            .collect::<HashMap<_, _>>();

        // For spaces with an initial proposal, get the space ID from the import (if available)
        let space_ids = stream::iter(initial_proposals)
            .filter_map(|(space_address, proposal_processed)| async move {
                let ipfs_hash = proposal_processed.content_uri.replace("ipfs://", "");
                self.ipfs
                    .get::<grc20::Import>(&ipfs_hash, true)
                    .await
                    .ok()
                    .map(|import| {
                        (
                            space_address,
                            create_space_id(
                                &import.previous_network,
                                &import.previous_contract_address,
                            ),
                        )
                    })
            })
            .collect::<HashMap<_, _>>()
            .await;

        // Create the spaces
        let created_ids: Vec<_> = stream::iter(spaces_created)
            .then(|event| async {
                let space_id = space_ids
                    .get(&event.space_address)
                    .cloned()
                    .unwrap_or(create_space_id(network_ids::GEO, &event.dao_address));

                tracing::info!(
                    "Block #{} ({}): Creating space {}",
                    block.block_number,
                    block.timestamp,
                    space_id
                );

                self.kg
                    .create_space(Space {
                        id: space_id.to_string(),
                        network: network_ids::GEO.to_string(),
                        contract_address: event.space_address.to_string(),
                        dao_contract_address: event.dao_address.to_string(),
                        r#type: SpaceType::Public,
                        created_at: block.timestamp,
                        created_at_block: block.block_number,
                    })
                    .await?;

                anyhow::Ok(space_id)
            })
            .try_collect()
            .await
            .map_err(|err| HandlerError::Other(format!("{err:?}").into()))?;

        Ok(created_ids)
    }
}