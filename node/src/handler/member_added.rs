use futures::stream;
use kg_core::{models, pb::geo};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_member_added(
        &self,
        member_added: &geo::MemberAdded,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space = self
            .kg
            .get_space_by_voting_plugin_address(&member_added.main_voting_plugin_address)
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

        if let Some(space) = &space {
            let member = models::GeoAccount::new(member_added.member_address.clone());

            self.kg.add_member(&space.id, &member, &models::SpaceMember, block)
                .await
                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not add members for unknown space with voting_plugin_address = {}",
                block.block_number,
                block.timestamp,
                member_added.main_voting_plugin_address
            );
        }

        Ok(())
    }
}