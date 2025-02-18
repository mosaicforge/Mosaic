use sdk::{
    indexer_ids,
    mapping::{entity_node, query_utils::Query},
    models::{self, Account, Proposal, VoteCast},
    pb::geo,
};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_vote_cast(
        &self,
        vote: &geo::VoteCast,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        // // TODO: (optimization) Merge the two queries into one OR query
        // match join!(
        //     Space::find_by_voting_plugin_address(&self.neo4j, &vote.plugin_address),
        //     Space::find_by_member_access_plugin(&self.neo4j, &vote.plugin_address)
        // ) {
        //     // Space found
        //     (Ok(Some(_space)), Ok(_)) | (Ok(None), Ok(Some(_space))) => {
        //         let maybe_proposal = models::Proposal::find_by_id_and_address(
        //             &self.neo4j,
        //             &vote.onchain_proposal_id,
        //             &vote.plugin_address,
        //         )
        //         .await?;

        //         let maybe_account =
        //             entity_node::find_one(&self.neo4j, &Account::generate_id(&vote.voter))
        //                 .send()
        //                 .await?;

        //         match (maybe_proposal, maybe_account) {
        //             (Some(proposal), Some(account)) => {
        //                 VoteCast::new(
        //                     &account.id,
        //                     &proposal.id,
        //                     vote.vote_option
        //                         .try_into()
        //                         .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?,
        //                 )
        //                 .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
        //                 .send()
        //                 .await?;
        //             }
        //             // Proposal or account not found
        //             (Some(_), None) => {
        //                 tracing::warn!(
        //                     "Block #{} ({}): Matching account not found for vote cast",
        //                     block.block_number,
        //                     block.timestamp,
        //                 );
        //             }
        //             (None, _) => {
        //                 tracing::warn!(
        //                     "Block #{} ({}): Matching proposal not found for vote cast",
        //                     block.block_number,
        //                     block.timestamp,
        //                 );
        //             }
        //         }
        //     }
        //     // Space not found
        //     (Ok(None), Ok(None)) => {
        //         tracing::warn!(
        //             "Block #{} ({}): Matching space in Proposal not found for plugin address = {}",
        //             block.block_number,
        //             block.timestamp,
        //             checksum_address(&vote.plugin_address),
        //         );
        //     }
        //     // Errors
        //     (Err(e), _) | (_, Err(e)) => {
        //         return Err(HandlerError::from(e));
        //     }
        // };

        // let maybe_space_id = triple::find_many(&self.neo4j)
        //     .attribute_id(prop_filter::value_in(vec![
        //         indexer_ids::SPACE_VOTING_PLUGIN_ADDRESS.into(),
        //         indexer_ids::SPACE_MEMBER_PLUGIN_ADDRESS.into(),
        //     ]))
        //     .value(prop_filter::value(&vote.plugin_address))
        //     .space_id(prop_filter::value(indexer_ids::INDEXER_SPACE_ID))
        //     .send()
        //     .await?
        //     .into_iter()
        //     .map(|triple| triple.entity)
        //     .next();

        let proposal_id = Proposal::gen_id(&vote.plugin_address, &vote.onchain_proposal_id);

        let maybe_proposal = entity_node::find_one(&self.neo4j, &proposal_id)
            .send()
            .await?;

        let maybe_account = entity_node::find_one(&self.neo4j, Account::gen_id(&vote.voter))
            .send()
            .await?;

        match (maybe_proposal, maybe_account) {
            (Some(proposal), Some(account)) => {
                VoteCast::new(
                    &account.id,
                    &proposal.id,
                    vote.vote_option
                        .try_into()
                        .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?,
                )
                .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
                .send()
                .await?;
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
                    "Block #{} ({}): Matching proposal {proposal_id} not found for vote cast",
                    block.block_number,
                    block.timestamp,
                );
            }
        }

        Ok(())
    }
}
