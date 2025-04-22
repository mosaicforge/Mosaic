use grc20_core::{
    block::BlockMetadata,
    indexer_ids,
    mapping::{attributes, query_utils::Query, Attributes},
    network_ids,
    pb::{self, geo},
};
use grc20_sdk::models::{account, space, SpaceGovernanceType};

use web3_utils::checksum_address;

use super::{handler::HandlerError, Edit, EventHandler};

impl EventHandler {
    /// Handles `GeoSpaceCreated` events.
    pub async fn handle_space_created(
        &self,
        space_created: &geo::GeoSpaceCreated,
        edits_published: &[(geo::EditPublished, Vec<Edit>)],
        block: &BlockMetadata,
    ) -> Result<String, HandlerError> {
        let maybe_initial_proposal = edits_published.iter().find(|proposal| {
            checksum_address(&proposal.0.plugin_address)
                == checksum_address(&space_created.space_address)
        });

        let maybe_existing_space_id = match maybe_initial_proposal {
            Some(initial_proposal) => {
                let bytes = self
                    .ipfs
                    .get_bytes(&initial_proposal.0.content_uri.replace("ipfs://", ""), true)
                    .await?;

                if let Ok(metadata) = ipfs::deserialize::<pb::ipfs::IpfsMetadata>(&bytes) {
                    match metadata.r#type() {
                        pb::ipfs::ActionType::ImportSpace => {
                            let import = ipfs::deserialize::<pb::ipfs::Import>(&bytes)?;

                            tracing::info!(
                                "Block #{} ({}): Found import for space {} (derived id: {})",
                                block.block_number,
                                block.timestamp,
                                checksum_address(&space_created.space_address),
                                space::new_id(
                                    &import.previous_network,
                                    &import.previous_contract_address,
                                )
                            );

                            Some(space::new_id(
                                &import.previous_network,
                                &import.previous_contract_address,
                            ))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            None => None,
        };

        let space_id = maybe_existing_space_id
            .unwrap_or_else(|| space::new_id(network_ids::GEO, &space_created.dao_address));

        tracing::info!(
            "Block #{} ({}): Creating space {}",
            block.block_number,
            block.timestamp,
            space_id
        );

        space::builder(&space_id, &space_created.dao_address)
            .network(network_ids::GEO.to_string())
            .space_plugin_address(&space_created.space_address)
            .build()
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await?;

        Ok(space_id)
    }

    pub async fn handle_personal_space_created(
        &self,
        personal_space_created: &geo::GeoPersonalSpaceAdminPluginCreated,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space_id = space::new_id(network_ids::GEO, &personal_space_created.dao_address);

        attributes::insert_one(
            &self.neo4j,
            block,
            &space_id,
            indexer_ids::INDEXER_SPACE_ID,
            "0",
            Attributes::default()
                .attribute((
                    indexer_ids::SPACE_GOVERNANCE_TYPE,
                    SpaceGovernanceType::Personal,
                ))
                .attribute((
                    indexer_ids::SPACE_PERSONAL_PLUGIN_ADDRESS,
                    checksum_address(&personal_space_created.personal_admin_address),
                )),
        )
        .send()
        .await?;

        // Add initial editors to the personal space
        let editor = account::new(personal_space_created.initial_editor.clone());

        editor
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await?;

        tracing::info!(
            "Block #{} ({}): Created personal admin space plugin for space {} with initial editor {}",
            block.block_number,
            block.timestamp,
            space_id,
            account::new_id(&personal_space_created.initial_editor),
        );

        Ok(())
    }

    pub async fn handle_governance_plugin_created(
        &self,
        governance_plugin_created: &geo::GeoGovernancePluginCreated,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space_id = space::new_id(network_ids::GEO, &governance_plugin_created.dao_address);

        attributes::insert_one(
            &self.neo4j,
            block,
            &space_id,
            indexer_ids::INDEXER_SPACE_ID,
            "0",
            Attributes::default()
                .attribute((
                    indexer_ids::SPACE_VOTING_PLUGIN_ADDRESS,
                    checksum_address(&governance_plugin_created.main_voting_address),
                ))
                .attribute((
                    indexer_ids::SPACE_MEMBER_PLUGIN_ADDRESS,
                    checksum_address(&governance_plugin_created.member_access_address),
                )),
        )
        .send()
        .await?;

        tracing::info!(
            "Block #{} ({}): Creating governance plugin for space {}",
            block.block_number,
            block.timestamp,
            space_id
        );

        Ok(())
    }
}
