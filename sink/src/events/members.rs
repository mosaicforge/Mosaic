use grc20_core::{
    block::BlockMetadata, indexer_ids, mapping::query_utils::Query, network_ids, pb::geo,
};
use grc20_sdk::models::{account, space, SpaceMember};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_member_added(
        &self,
        member_added: &geo::MemberAdded,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space_id = space::new_id(network_ids::GEO, &member_added.dao_address);

        let member = account::new(member_added.member_address.clone());
        let member_rel = SpaceMember::new(member.id(), &space_id);

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

        Ok(())
    }

    pub async fn handle_member_removed(
        &self,
        member_removed: &geo::MemberRemoved,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space_id = space::new_id(network_ids::GEO, &member_removed.dao_address);

        SpaceMember::remove(
            &self.neo4j,
            block,
            &account::new_id(&member_removed.member_address),
            &space_id,
        )
        .await?;

        Ok(())
    }
}
