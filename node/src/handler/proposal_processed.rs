use futures::{stream, StreamExt, TryStreamExt};
use ipfs::deserialize;
use kg_core::{
    models::{self, EditProposal},
    pb::{geo, grc20},
};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_proposals_processed(
        &self,
        proposals_processed: &[geo::ProposalProcessed],
        created_space_ids: &[String],
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let proposals = stream::iter(proposals_processed)
            .then(|proposal| async {
                let edits = self.fetch_edit_proposals(proposal).await?;
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

    async fn fetch_edit_proposals(
        &self,
        proposal_processed: &geo::ProposalProcessed,
    ) -> Result<Vec<EditProposal>, HandlerError> {
        let space = if let Some(space) = self
            .kg
            .get_space_by_space_plugin_address(&proposal_processed.plugin_address)
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?
        {
            space
        } else {
            tracing::warn!(
                "Matching space in Proposal not found for plugin address {}",
                proposal_processed.plugin_address
            );
            return Ok(vec![]);
        };

        let bytes = self
            .ipfs
            .get_bytes(&proposal_processed.content_uri.replace("ipfs://", ""), true)
            .await?;

        let metadata = deserialize::<grc20::Metadata>(&bytes)?;
        match metadata.r#type() {
            grc20::ActionType::AddEdit => {
                let edit = deserialize::<grc20::Edit>(&bytes)?;
                Ok(vec![EditProposal {
                    name: edit.name,
                    proposal_id: edit.id,
                    space: space.id,
                    space_address: space
                        .space_plugin_address
                        .expect("Space plugin address not found"),
                    creator: edit.authors[0].clone(),
                    ops: edit.ops,
                }])
            }
            grc20::ActionType::ImportSpace => {
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
                        space: space.id.clone(),
                        space_address: space
                            .space_plugin_address
                            .clone()
                            .expect("Space plugin address not found"),
                        creator: edit.authors[0].clone(),
                        ops: edit.ops,
                    })
                    .collect())
            }
            _ => Err(HandlerError::Other("Invalid metadata".into())),
        }
    }
}
