use std::collections::HashMap;

use crate::kg::mapping::Node;
use futures::{stream, StreamExt, TryStreamExt};
use kg_core::{
    ids,
    models::{self, GeoAccount, Space, SpaceType},
    network_ids,
    pb::{geo, grc20},
    system_ids,
};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    /// Handles `GeoSpaceCreated` events. `ProposalProcessed` events are used to determine
    /// the space's ID in cases where the space is imported.
    ///
    /// The method returns the IDs of the spaces that were successfully created.
    pub async fn handle_spaces_created(
        &self,
        spaces_created: &[geo::GeoSpaceCreated],
        proposals_processed: &[geo::ProposalProcessed],
        block: &models::BlockMetadata,
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
                            ids::create_space_id(
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
                    .unwrap_or(ids::create_space_id(network_ids::GEO, &event.dao_address));

                tracing::info!(
                    "Block #{} ({}): Creating space {}",
                    block.block_number,
                    block.timestamp,
                    space_id
                );

                self.kg
                    .upsert_node(
                        system_ids::INDEXER_SPACE_ID,
                        block,
                        Node::new(
                            space_id.to_string(),
                            Space {
                                id: space_id.to_string(),
                                network: network_ids::GEO.to_string(),
                                dao_contract_address: checksum_address(&event.dao_address, None),
                                space_plugin_address: Some(checksum_address(
                                    &event.space_address,
                                    None,
                                )),
                                r#type: SpaceType::Public,
                                ..Default::default()
                            },
                        )
                        .with_type(system_ids::INDEXED_SPACE),
                    )
                    .await?;

                anyhow::Ok(space_id)
            })
            .try_collect()
            .await
            .map_err(|err| HandlerError::Other(format!("{err:?}").into()))?;

        Ok(created_ids)
    }

    pub async fn handle_personal_space_created(
        &self,
        personal_space_created: &geo::GeoPersonalSpaceAdminPluginCreated,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space = self
            .kg
            .get_space_by_dao_address(&personal_space_created.dao_address)
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

        if let Some(space) = &space {
            self.kg
                .upsert_node(
                    system_ids::INDEXER_SPACE_ID,
                    block,
                    Node::new(
                        space.id.clone(),
                        Space {
                            r#type: SpaceType::Personal,
                            personal_space_admin_plugin: Some(checksum_address(
                                &personal_space_created.personal_admin_address,
                                None,
                            )),
                            ..space.clone()
                        },
                    )
                    .with_type(system_ids::INDEXED_SPACE),
                )
                .await
                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

            // // Add initial editors to the personal space
            let editor = GeoAccount::new(personal_space_created.initial_editor.clone());

            self.kg
                .add_editor(&space.id, &editor, &models::SpaceEditor, block)
                .await
                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

            tracing::info!(
                "Block #{} ({}): Creating personal admin space plugin for space {} with initial editor {}",
                block.block_number,
                block.timestamp,
                space.id,
                editor.id,
            );
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not create personal admin space plugin for unknown space with dao_address = {}",
                block.block_number,
                block.timestamp,
                personal_space_created.dao_address
            );
        }

        Ok(())
    }

    pub async fn handle_governance_plugin_created(
        &self,
        governance_plugin_created: &geo::GeoGovernancePluginCreated,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space = self
            .kg
            .get_space_by_dao_address(&governance_plugin_created.dao_address)
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

        if let Some(space) = space {
            tracing::info!(
                "Block #{} ({}): Creating governance plugin for space {}",
                block.block_number,
                block.timestamp,
                space.id
            );

            self.kg
                .upsert_node(
                    system_ids::INDEXER_SPACE_ID,
                    block,
                    Node::new(
                        space.id.clone(),
                        Space {
                            voting_plugin_address: Some(checksum_address(
                                &governance_plugin_created.main_voting_address,
                                None,
                            )),
                            member_access_plugin: Some(checksum_address(
                                &governance_plugin_created.member_access_address,
                                None,
                            )),
                            ..space
                        },
                    )
                    .with_type(system_ids::INDEXED_SPACE),
                )
                .await
                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not create governance plugin for unknown space with dao_address = {}",
                block.block_number,
                block.timestamp,
                checksum_address(&governance_plugin_created.dao_address, None)
            );
        }

        Ok(())
    }
}
