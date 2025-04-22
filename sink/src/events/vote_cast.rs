use grc20_core::{block::BlockMetadata, indexer_ids, mapping::query_utils::Query, pb::geo};
use grc20_sdk::models::{account, Proposal, VoteCast};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_vote_cast(
        &self,
        vote: &geo::VoteCast,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let proposal_id = Proposal::gen_id(&vote.plugin_address, &vote.onchain_proposal_id);
        let account_id = account::new_id(&vote.voter);

        VoteCast::new(
            &account_id,
            &proposal_id,
            vote.vote_option
                .try_into()
                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?,
        )
        .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
        .send()
        .await?;

        Ok(())
    }
}
