use kg_core::{models, pb::geo};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_editor_removed(
        &self,
        editor_removed: &geo::EditorRemoved,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space = self
            .kg
            .get_space_by_dao_address(&editor_removed.dao_address)
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;
        
        if let Some(space) = space {
            self.kg.remove_editor(
                &models::GeoAccount::id_from_address(&editor_removed.editor_address),
                &space.id, 
                block
            )
                .await
                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not remove editor for unknown space with dao_address = {}",
                block.block_number,
                block.timestamp,
                editor_removed.dao_address
            );
        }

        Ok(())
    }
}