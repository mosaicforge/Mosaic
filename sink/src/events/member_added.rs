use grc20_core::{block::BlockMetadata, indexer_ids, mapping::query_utils::Query, pb::geo};
use grc20_sdk::models::{account, space, SpaceMember};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_member_added(
        &self,
        member_added: &geo::MemberAdded,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        // match try_join!(
        //     space::find_by_voting_plugin_address(
        //         &self.neo4j,
        //         &member_added.main_voting_plugin_address
        //     ),
        //     space::find_by_personal_plugin_address(
        //         &self.neo4j,
        //         &member_added.main_voting_plugin_address
        //     )
        // )? {
        //     // Space found
        //     (Some(space), _) | (None, Some(space)) => {
        //         let member = account::new(member_added.member_address.clone());
        //         let member_rel = SpaceMember::new(&member.id, &space.id);

        //         // Add geo account
        //         member.insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, 0)
        //             .send()
        //             .await?;

        //         // Add space member relation
        //         member_rel
        //             .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, 0)
        //             .send()
        //             .await?;
        //     }
        //     // Space not found
        //     (None, None) => {
        //         tracing::warn!(
        //             "Block #{} ({}): Could not add members for unknown space with voting_plugin_address = {}",
        //             block.block_number,
        //             block.timestamp,
        //             member_added.main_voting_plugin_address
        //         );
        //     }
        // };

        if let Some(space) =
            space::find_entity_by_dao_address(&self.neo4j, &member_added.dao_address).await?
        {
            let member = account::new(member_added.member_address.clone());
            let member_rel = SpaceMember::new(&member.id, &space.id);

            // Add geo account
            member
                .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
                .send()
                .await?;

            // Add space member relation
            member_rel
                .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
                .send()
                .await?;
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not add members for unknown space with dao_address = {}",
                block.block_number,
                block.timestamp,
                member_added.dao_address
            );
        }

        Ok(())
    }
}
