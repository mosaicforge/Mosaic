use sdk::{models, pb::geo, system_ids};

use crate::kg::mapping::Node;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_proposal_executed(
        &self,
        proposal_executed: &geo::ProposalExecuted,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let proposal = self
            .kg
            .get_proposal_by_id_and_address(
                &proposal_executed.proposal_id,
                &proposal_executed.plugin_address,
            )
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

        if let Some(mut proposal) = proposal {
            proposal.status = models::ProposalStatus::Executed;
            self.kg
                .upsert_node(
                    
                    block,
                    Node::new(&proposal.id, system_ids::INDEXER_SPACE_ID, proposal.clone()),
                )
                .await
                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

            tracing::info!(
                "Block #{} ({}): Proposal {} executed",
                block.block_number,
                block.timestamp,
                proposal_executed.proposal_id
            );
        } else {
            tracing::warn!(
                "Block #{} ({}): Proposal {} not found",
                block.block_number,
                block.timestamp,
                proposal_executed.proposal_id
            );
        };

        Ok(())
    }
}
