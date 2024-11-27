use kg_core::{models, pb::geo};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_editor_added(
        &self,
        editor_added: &geo::EditorAdded,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space = self
            .kg
            .get_space_by_voting_plugin_address(&editor_added.main_voting_plugin_address)
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;
        
        if let Some(space) = space {
            let editor = models::GeoAccount::new(editor_added.editor_address.clone());
            self.kg.add_editor(&space.id, &editor, &models::SpaceEditor, block)
                .await
                .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not add editor for unknown space with voting_plugin_address = {}",
                block.block_number,
                block.timestamp,
                editor_added.main_voting_plugin_address
            );
        }

        Ok(())
    }
}
