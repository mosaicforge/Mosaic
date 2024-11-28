use futures::join;
use ipfs::deserialize;
use kg_core::{
    ids,
    models::{self, EditorshipProposal, GeoAccount, MembershipProposal, Proposal},
    pb::{self, geo, grc20},
    system_ids::{self, INDEXER_SPACE_ID},
};
use web3_utils::checksum_address;

use crate::kg::mapping::{Node, Relation};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_proposal_created(
        &self,
        proposal_created: &geo::ProposalCreated,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        match join!(
            self.kg
                .get_space_by_voting_plugin_address(&proposal_created.plugin_address),
            self.kg
                .get_space_by_member_access_plugin(&proposal_created.plugin_address)
        ) {
            // Space found
            (Ok(Some(space)), Ok(_)) | (Ok(None), Ok(Some(space))) => {
                let bytes = self
                    .ipfs
                    .get_bytes(&&proposal_created.metadata_uri.replace("ipfs://", ""), true)
                    .await?;

                let metadata = deserialize::<pb::ipfs::IpfsMetadata>(&bytes)?;

                match metadata.r#type() {
                    pb::ipfs::ActionType::AddEdit => todo!(),
                    pb::ipfs::ActionType::AddSubspace | pb::ipfs::ActionType::RemoveSubspace => {
                        let subspace_proposal = deserialize::<pb::ipfs::Subspace>(&bytes)?;
                        
                        self.kg.upsert_node(
                            INDEXER_SPACE_ID,
                            block, 
                            Node::new(
                                subspace_proposal.id.clone(),
                                models::SubspaceProposal {
                                    proposal: Proposal {
                                        id: subspace_proposal.id.clone(),
                                        onchain_proposal_id: proposal_created.proposal_id.clone(),
                                        proposal_type: metadata.r#type().try_into().map_err(|e: String| HandlerError::Other(e.into()))?,
                                        status: models::ProposalStatus::Proposed,
                                        plugin_address: checksum_address(&proposal_created.plugin_address, None),
                                        start_time: proposal_created.start_time.parse().map_err(|e| HandlerError::Other(format!("{e:?}").into()))?,
                                        end_time: proposal_created.end_time.parse().map_err(|e| HandlerError::Other(format!("{e:?}").into()))?,
                                    },
                                    proposal_type: subspace_proposal.r#type().try_into().map_err(|e: String| HandlerError::Other(e.into()))?,
                                },
                            )
                            .with_type(system_ids::PROPOSAL_TYPE)
                            .with_type(system_ids::SUBSPACE_PROPOSAL_TYPE),
                        )
                        .await
                        .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

                        // Try to get the subspace
                        let subspace = if let Some(subspace) = self.kg.get_space_by_dao_address(&subspace_proposal.subspace)
                            .await
                            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))? {
                            subspace
                            } else {
                                tracing::warn!(
                                    "Block #{} ({}): Failed to get space for subspace DAO address = {}",
                                    block.block_number,
                                    block.timestamp,
                                    checksum_address(&subspace_proposal.subspace, None)
                                );
                                return Ok(());
                            };

                        // Create relation between the proposal and the subspace
                        self.kg
                            .upsert_relation(
                                INDEXER_SPACE_ID,
                                block,
                                Relation::new(
                                    &ids::create_geo_id(),
                                    &subspace_proposal.id,
                                    &subspace.id,
                                    system_ids::PROPOSED_SUBSPACE,
                                    models::ProposedSubspace,
                                ),
                            )
                            .await
                            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

                        // Create relation between the proposal and the space
                        self.kg
                            .upsert_relation(
                                INDEXER_SPACE_ID,
                                block,
                                Relation::new(
                                    &ids::create_geo_id(),
                                    &space.id,
                                    &subspace_proposal.id,
                                    system_ids::PROPOSALS,
                                    models::Proposals,
                                ),
                            )
                            .await
                            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

                        tracing::info!(
                            "Block #{} ({}): Added subspace proposal {} for space {}",
                            block.block_number,
                            block.timestamp,
                            subspace_proposal.id,
                            space.id
                        );
                    }
                    pb::ipfs::ActionType::AddEditor | pb::ipfs::ActionType::RemoveEditor => {
                        let editor_proposal = deserialize::<pb::ipfs::Membership>(&bytes)?;

                        self.kg
                            .upsert_node(
                                INDEXER_SPACE_ID,
                                block,
                                Node::new(
                                    editor_proposal.id.clone(),
                                    EditorshipProposal {
                                        proposal: Proposal {
                                            id: editor_proposal.id.clone(),
                                            onchain_proposal_id: proposal_created
                                                .proposal_id
                                                .clone(),
                                            proposal_type: metadata.r#type().try_into().map_err(
                                                |e: String| HandlerError::Other(e.into()),
                                            )?,
                                            status: models::ProposalStatus::Proposed,
                                            plugin_address: checksum_address(
                                                &proposal_created.plugin_address,
                                                None,
                                            ),
                                            start_time: proposal_created
                                                .start_time
                                                .parse()
                                                .map_err(|e| {
                                                    HandlerError::Other(format!("{e:?}").into())
                                                })?,
                                            end_time: proposal_created.end_time.parse().map_err(
                                                |e| HandlerError::Other(format!("{e:?}").into()),
                                            )?,
                                        },
                                        proposal_type: editor_proposal
                                            .r#type()
                                            .try_into()
                                            .map_err(|e: String| HandlerError::Other(e.into()))?,
                                    },
                                )
                                .with_type(system_ids::PROPOSAL_TYPE)
                                .with_type(system_ids::EDITORSHIP_PROPOSAL_TYPE),    
                            )
                            .await
                            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

                        // Create relation between the proposal and the editor
                        self.kg
                            .upsert_relation(
                                INDEXER_SPACE_ID,
                                block,
                                Relation::new(
                                    &ids::create_geo_id(),
                                    &editor_proposal.id,
                                    &GeoAccount::id_from_address(&editor_proposal.user),
                                    system_ids::PROPOSED_ACCOUNT,
                                    models::ProposedAccount,
                                ),
                            )
                            .await
                            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

                        // Create relation between the space and the proposal
                        self.kg
                            .upsert_relation(
                                INDEXER_SPACE_ID,
                                block,
                                Relation::new(
                                    &ids::create_geo_id(),
                                    &space.id,
                                    &editor_proposal.id,
                                    system_ids::PROPOSALS,
                                    models::Proposals,
                                ),
                            )
                            .await
                            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

                        tracing::info!(
                            "Block #{} ({}): Added editorship proposal {} for space {}",
                            block.block_number,
                            block.timestamp,
                            editor_proposal.id,
                            space.id
                        );
                    }
                    pb::ipfs::ActionType::AddMember | pb::ipfs::ActionType::RemoveMember => {
                        let member_proposal = deserialize::<pb::ipfs::Membership>(&bytes)?;

                        self.kg
                            .upsert_node(
                                INDEXER_SPACE_ID,
                                block,
                                Node::new(
                                    member_proposal.id.clone(),
                                    MembershipProposal {
                                        proposal: Proposal {
                                            id: member_proposal.id.clone(),
                                            onchain_proposal_id: proposal_created
                                                .proposal_id
                                                .clone(),
                                            proposal_type: metadata.r#type().try_into().map_err(
                                                |e: String| HandlerError::Other(e.into()),
                                            )?,
                                            status: models::ProposalStatus::Proposed,
                                            plugin_address: checksum_address(
                                                &proposal_created.plugin_address,
                                                None,
                                            ),
                                            start_time: proposal_created
                                                .start_time
                                                .parse()
                                                .map_err(|e| {
                                                    HandlerError::Other(format!("{e:?}").into())
                                                })?,
                                            end_time: proposal_created.end_time.parse().map_err(
                                                |e| HandlerError::Other(format!("{e:?}").into()),
                                            )?,
                                        },
                                        proposal_type: member_proposal
                                            .r#type()
                                            .try_into()
                                            .map_err(|e: String| HandlerError::Other(e.into()))?,
                                    },
                                )
                                .with_type(system_ids::PROPOSAL_TYPE)
                                .with_type(system_ids::MEMBERSHIP_PROPOSAL_TYPE),
                            )
                            .await
                            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

                        // Create relation between the proposal and the member
                        self.kg
                            .upsert_relation(
                                INDEXER_SPACE_ID,
                                block,
                                Relation::new(
                                    &ids::create_geo_id(),
                                    &member_proposal.id,
                                    &GeoAccount::id_from_address(&member_proposal.user),
                                    system_ids::PROPOSED_ACCOUNT,
                                    models::ProposedAccount,
                                ),
                            )
                            .await
                            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

                        // Create relation between the space and the proposal
                        self.kg
                            .upsert_relation(
                                INDEXER_SPACE_ID,
                                block,
                                Relation::new(
                                    &ids::create_geo_id(),
                                    &space.id,
                                    &member_proposal.id,
                                    system_ids::PROPOSALS,
                                    models::Proposals,
                                ),
                            )
                            .await
                            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

                        tracing::info!(
                            "Block #{} ({}): Added membership proposal {} for space {}",
                            block.block_number,
                            block.timestamp,
                            member_proposal.id,
                            space.id
                        );
                    }
                    pb::ipfs::ActionType::Empty => (),
                    action_type => {
                        return Err(HandlerError::Other(
                            format!("Invalid proposal action type {action_type:?}").into(),
                        ))
                    }
                }
            }
            // Space not found
            (Ok(None), Ok(None)) => {
                tracing::warn!(
                    "Block #{} ({}): Matching space in Proposal not found for plugin address = {}",
                    block.block_number,
                    block.timestamp,
                    checksum_address(&proposal_created.plugin_address, None)
                );
            }
            // Errors
            (Err(e), _) | (_, Err(e)) => {
                return Err(HandlerError::Other(format!("{e:?}").into()));
            }
        };

        Ok(())
    }
}
