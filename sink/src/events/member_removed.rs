use sdk::{models, pb::geo};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_member_removed(
        &self,
        member_removed: &geo::MemberRemoved,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space = self
            .kg
            .get_space_by_dao_address(&member_removed.dao_address)
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

        if let Some(space) = space {
            self.kg
                .remove_member(
                    &models::GeoAccount::id_from_address(&member_removed.member_address),
                    &space.id,
                    block,
                )
                .await
                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not remove member for unknown space with dao_address = {}",
                block.block_number,
                block.timestamp,
                member_removed.dao_address
            );
        }

        Ok(())
    }
}
