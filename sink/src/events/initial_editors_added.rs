use futures::{stream, StreamExt, TryStreamExt};
use sdk::{models, pb::geo};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_initial_space_editors_added(
        &self,
        initial_editor_added: &geo::InitialEditorAdded,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space = self
            .kg
            .get_space_by_voting_plugin_address(&initial_editor_added.plugin_address)
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

        if let Some(space) = &space {
            stream::iter(&initial_editor_added.addresses)
                .map(Result::<_, HandlerError>::Ok)
                .try_for_each(|editor| async move {
                    let editor = models::GeoAccount::new(editor.clone());
                    self.kg
                        .add_editor(&space.id, &editor, &models::SpaceEditor, block)
                        .await
                        .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

                    Ok(())
                })
                .await?;

            tracing::info!(
                "Block #{} ({}): Added {} initial editors to space {}",
                block.block_number,
                block.timestamp,
                initial_editor_added.addresses.len(),
                space.id
            );
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not add initial editors for unknown space with plugin_address = {}",
                block.block_number,
                block.timestamp,
                initial_editor_added.plugin_address
            );
        }

        Ok(())
    }
}
