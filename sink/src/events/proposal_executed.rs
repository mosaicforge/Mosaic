use sdk::{
    indexer_ids,
    mapping::{
        entity_node, query_utils::prop_filter, relation_node, triple, Query, Triple
    },
    models::{self, proposal::ProposalStatus, Proposal, Proposals},
    pb::geo,
    system_ids,
};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_proposal_executed(
        &self,
        proposal_executed: &geo::ProposalExecuted,
        block: &models::BlockMetadata,
        index: usize,
    ) -> Result<(), HandlerError> {
        let plugin_address = checksum_address(&proposal_executed.plugin_address);
        let proposal_id = Proposal::gen_id(
            &proposal_executed.plugin_address, 
            &proposal_executed.proposal_id,
        );

        let maybe_proposal = entity_node::find_one(&self.neo4j, &proposal_id)
            .send()
            .await?;

        // Find space
        let maybe_space_id = triple::find_many(&self.neo4j)
            .attribute_id(prop_filter::value_in(vec![
                indexer_ids::SPACE_VOTING_PLUGIN_ADDRESS.into(),
                indexer_ids::SPACE_MEMBER_PLUGIN_ADDRESS.into(),
                indexer_ids::SPACE_PERSONAL_PLUGIN_ADDRESS.into(),
            ]))
            .value(prop_filter::value(&plugin_address))
            .space_id(prop_filter::value(indexer_ids::INDEXER_SPACE_ID))
            .send()
            .await?
            .into_iter()
            .map(|triple| triple.entity)
            .next();

        match (maybe_space_id, maybe_proposal) {
            (Some(space_id), Some(proposal)) => {
                // Update proposal status
                Triple::new(
                    &proposal.id,
                    "status",  // TODO: Change to GRC20 id
                    ProposalStatus::Executed,
                )
                .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
                .send()
                .await?;

                // let proposals_relation = relation_node::find_one(
                //     &self.neo4j,
                //     Proposals::gen_id(
                //         &space_id, 
                //         &proposal.id),
                //     indexer_ids::INDEXER_SPACE_ID,
                //     None,
                // )
                // .send()
                // .await?;

                // tracing::info!(
                //     "Block #{} ({}): Proposal {proposal_id} executed for space {space_id}",
                //     block.block_number,
                //     block.timestamp,
                // );        
    
                // // Update PROPOSALS relation between space and proposal
                // if let Some(proposals_relation) = proposals_relation {
                //     Triple::new(
                //         proposals_relation.id,
                //         system_ids::RELATION_INDEX,
                //         format!("{}:{}", block.block_number, index),
                //     )
                //     .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
                //     .send()
                //     .await?;
                // } else {
                //     tracing::warn!(
                //         "Block #{} ({}): Proposal {proposal_id} executed, but no relation found with space {space_id}",
                //         block.block_number,
                //         block.timestamp,
                //     );
                // }
            }
            (None, _) => {
                tracing::warn!(
                    "Block #{} ({}): Proposal space not found with plugin address {plugin_address}",
                    block.block_number,
                    block.timestamp,
                );
            }
            (_, None) => {
                tracing::warn!(
                    "Block #{} ({}): Proposal {proposal_id} not found for space plugin address {plugin_address}",
                    block.block_number,
                    block.timestamp,
                );
            }
        }

        Ok(())
    }
}
