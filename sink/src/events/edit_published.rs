use futures::{stream, StreamExt, TryStreamExt};
use ipfs::deserialize;
use sdk::{
    mapping::Node, models::{self, EditProposal}, pb::{self, geo, grc20}
};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_edits_published(
        &self,
        edits_published: &[geo::EditPublished],
        _created_space_ids: &[String],
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let proposals = stream::iter(edits_published)
            .then(|proposal| async {
                let edits = self.fetch_edit(proposal).await?;
                anyhow::Ok(edits)
            })
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))? // TODO: Convert anyhow::Error to HandlerError properly
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        // TODO: Create "synthetic" proposals for newly created spaces and
        // personal spaces

        stream::iter(proposals)
            .map(Ok) // Need to wrap the proposal in a Result to use try_for_each
            .try_for_each(|proposal| async {
                tracing::info!(
                    "Block #{} ({}): Creating edit proposal {}",
                    block.block_number,
                    block.timestamp,
                    proposal.proposal_id
                );
                self.kg.process_edit(proposal).await
            })
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

        Ok(())
    }

    async fn fetch_edit(
        &self,
        edit: &geo::EditPublished,
    ) -> Result<Vec<EditProposal>, HandlerError> {
        let space = if let Some(space) = self
            .kg
            .get_space_by_space_plugin_address(&edit.plugin_address)
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?
        {
            space
        } else {
            tracing::warn!(
                "Matching space in edit not found for plugin address {}",
                edit.plugin_address
            );
            return Ok(vec![]);
        };

        let bytes = self
            .ipfs
            .get_bytes(&edit.content_uri.replace("ipfs://", ""), true)
            .await?;

        let metadata = if let Ok(metadata) = deserialize::<pb::ipfs::IpfsMetadata>(&bytes) {
            metadata
        } else {
            tracing::warn!(
                "Invalid metadata for edit {}",
                edit.content_uri
            );
            return Ok(vec![]);
        };

        match metadata.r#type() {
            pb::ipfs::ActionType::AddEdit => {
                let edit = deserialize::<grc20::Edit>(&bytes)?;
                Ok(vec![EditProposal {
                    name: edit.name,
                    proposal_id: edit.id,
                    space: space.id().to_string(),
                    space_address: space
                        .attributes()
                        .space_plugin_address
                        .clone()
                        .expect("Space plugin address not found"),
                    creator: edit.authors[0].clone(),
                    ops: edit.ops,
                }])
            }
            pb::ipfs::ActionType::ImportSpace => {
                let import = deserialize::<grc20::Import>(&bytes)?;
                let edits = stream::iter(import.edits)
                    .map(|edit| async move {
                        let hash = edit.replace("ipfs://", "");
                        self.ipfs.get::<grc20::ImportEdit>(&hash, true).await
                    })
                    .buffer_unordered(10)
                    .try_collect::<Vec<_>>()
                    .await?;

                Ok(edits
                    .into_iter()
                    .map(|edit| EditProposal {
                        name: edit.name,
                        proposal_id: edit.id,
                        space: space.id().to_string(),
                        space_address: space
                            .attributes()
                            .space_plugin_address
                            .clone()
                            .expect("Space plugin address not found"),
                        creator: edit.authors[0].clone(),
                        ops: edit.ops,
                    })
                    .collect())
            }
            _ => Ok(vec![]),
        }
    }
}
