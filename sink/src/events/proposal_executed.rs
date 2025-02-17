use sdk::{
    indexer_ids,
    mapping::{
        entity_node, query_utils::prop_filter, relation_node, AttributeFilter, Query, Triple,
    },
    models::{self, Proposals},
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

        models::Proposal::set_status(
            &self.neo4j,
            block,
            &proposal_executed.proposal_id,
            models::proposal::ProposalStatus::Executed,
        )
        .await?;

        // Find space
        let space = entity_node::find_many(&self.neo4j)
            .attribute(
                AttributeFilter::new(indexer_ids::SPACE_PLUGIN_ADDRESS)
                    .value(prop_filter::value(&plugin_address))
                    .space_id(prop_filter::value(indexer_ids::INDEXER_SPACE_ID)),
            )
            .send()
            .await?
            .into_iter()
            .next();

        if let Some(space) = space {
            let proposals_relation = relation_node::find_one(
                &self.neo4j,
                Proposals::gen_id(&space.id, &proposal_executed.proposal_id),
                indexer_ids::INDEXER_SPACE_ID,
                None,
            )
            .send()
            .await?;

            if let Some(proposals_relation) = proposals_relation {
                Triple::new(
                    proposals_relation.id,
                    system_ids::RELATION_INDEX,
                    format!("{}:{}", block.block_number, index),
                )
                .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
                .send()
                .await?;
            } else {
                tracing::warn!(
                    "Block #{} ({}): Proposal {} executed, but no relation found with space {}",
                    block.block_number,
                    block.timestamp,
                    proposal_executed.proposal_id,
                    space.id
                );
            }
        } else {
            tracing::warn!(
                "Block #{} ({}): Proposal {} executed, but space not found with plugin address {plugin_address}",
                block.block_number,
                block.timestamp,
                proposal_executed.proposal_id
            );
        }

        Ok(())
    }
}
