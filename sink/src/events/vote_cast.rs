use futures::join;
use sdk::{
    ids, models,
    pb::geo,
    system_ids::{self, INDEXER_SPACE_ID},
};
use web3_utils::checksum_address;

use crate::{kg::mapping::Relation, neo4j_utils::Neo4jExt};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_vote_cast(
        &self,
        vote: &geo::VoteCast,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        match join!(
            self.kg
                .get_space_by_voting_plugin_address(&vote.plugin_address),
            self.kg
                .get_space_by_member_access_plugin(&vote.plugin_address)
        ) {
            // Space found
            (Ok(Some(space)), Ok(_)) | (Ok(None), Ok(Some(space))) => {
                let proposal = self.kg.neo4j
                    .find_one::<models::Proposal>(neo4rs::query(&format!(
                        "MATCH (p:`{PROPOSAL_TYPE}` {{onchain_proposal_id: $onchain_proposal_id}})<-[:`{PROPOSALS}`]-(:`{INDEXED_SPACE}` {{id: $space_id}}) RETURN p",
                        PROPOSAL_TYPE = system_ids::PROPOSAL_TYPE,
                        PROPOSALS = system_ids::PROPOSALS,
                        INDEXED_SPACE = system_ids::INDEXED_SPACE,
                    ))
                    .param("onchain_proposal_id", vote.onchain_proposal_id.clone())
                    .param("space_id", space.id))
                    .await
                    .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

                let account = self
                    .kg
                    .neo4j
                    .find_one::<models::GeoAccount>(
                        neo4rs::query(&format!(
                            "MATCH (a:`{ACCOUNT}` {{address: $address}}) RETURN a",
                            ACCOUNT = system_ids::GEO_ACCOUNT,
                        ))
                        .param("address", checksum_address(&vote.voter, None)),
                    )
                    .await
                    .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

                match (proposal, account) {
                    (Some(proposal), Some(account)) => {
                        let vote_cast = models::VoteCast {
                            id: ids::create_geo_id(),
                            vote_type: vote
                                .vote_option
                                .try_into()
                                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?,
                        };

                        self.kg
                            .upsert_relation(
                                block,
                                Relation::new(
                                    INDEXER_SPACE_ID,
                                    &vote_cast.id.clone(),
                                    &account.id,
                                    &proposal.id,
                                    system_ids::VOTE_CAST,
                                    vote_cast,
                                ),
                            )
                            .await
                            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;
                    }
                    // Proposal or account not found
                    (Some(_), None) => {
                        tracing::warn!(
                            "Block #{} ({}): Matching account not found for vote cast",
                            block.block_number,
                            block.timestamp,
                        );
                    }
                    (None, _) => {
                        tracing::warn!(
                            "Block #{} ({}): Matching proposal not found for vote cast",
                            block.block_number,
                            block.timestamp,
                        );
                    }
                }
            }
            // Space not found
            (Ok(None), Ok(None)) => {
                tracing::warn!(
                    "Block #{} ({}): Matching space in Proposal not found for plugin address = {}",
                    block.block_number,
                    block.timestamp,
                    vote.plugin_address,
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
