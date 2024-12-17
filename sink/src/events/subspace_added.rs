use futures::join;
use sdk::{models, pb::geo};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_subspace_added(
        &self,
        subspace_added: &geo::SubspaceAdded,
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        match join!(
            self.kg
                .get_space_by_space_plugin_address(&subspace_added.plugin_address),
            self.kg.get_space_by_dao_address(&subspace_added.subspace)
        ) {
            (Ok(Some(parent_space)), Ok(Some(subspace))) => {
                self.kg
                    .add_subspace(block, &parent_space.id(), &subspace.id())
                    .await
                    .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;
                // TODO: Convert anyhow::Error to HandlerError properly
            }
            (Ok(None), Ok(_)) => {
                tracing::warn!(
                    "Block #{} ({}): Could not create subspace: parent space with plugin_address = {} not found",
                    block.block_number,
                    block.timestamp,
                    subspace_added.plugin_address
                );
            }
            (Ok(Some(_)), Ok(None)) => {
                tracing::warn!(
                    "Block #{} ({}): Could not create subspace: space with dao_address = {} not found",
                    block.block_number,
                    block.timestamp,
                    subspace_added.plugin_address
                );
            }
            (Err(e), _) | (_, Err(e)) => {
                Err(HandlerError::from(e))?;
            }
        };

        Ok(())
    }
}
