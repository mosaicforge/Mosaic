use grc20_core::{
    block::BlockMetadata,
    indexer_ids,
    mapping::{Query, Triple},
    pb::geo,
};
use grc20_sdk::models::{proposal::ProposalStatus, Proposal};
use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_proposal_executed(
        &self,
        proposal_executed: &geo::ProposalExecuted,
        block: &BlockMetadata,
        _index: usize,
    ) -> Result<(), HandlerError> {
        let proposal_id = Proposal::gen_id(
            &proposal_executed.plugin_address,
            &proposal_executed.proposal_id,
        );

        // Update proposal status
        Triple::new(
            &proposal_id,
            "status", // TODO: Change to GRC20 id
            ProposalStatus::Executed,
        )
        .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
        .send()
        .await?;

        Ok(())
    }
}
