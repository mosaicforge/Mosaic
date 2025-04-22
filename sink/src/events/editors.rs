use futures::{stream, StreamExt, TryStreamExt};
use grc20_core::{block::BlockMetadata, indexer_ids, mapping::query_utils::Query, network_ids, pb::geo};
use grc20_sdk::models::{account, space, SpaceEditor};

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_editor_added(
        &self,
        editor_added: &geo::EditorAdded,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space_id = space::new_id(network_ids::GEO, &editor_added.dao_address);

        // Create editor account and space editor relation
        let editor = account::new(editor_added.editor_address.clone());
        let editor_relation = SpaceEditor::new(editor.id(), &space_id);

        // Insert editor account
        editor
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await?;

        // Insert space editor relation
        editor_relation
            .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
            .send()
            .await?;

        Ok(())
    }

    pub async fn handle_editor_removed(
        &self,
        editor_removed: &geo::EditorRemoved,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space_id = space::new_id(network_ids::GEO, &editor_removed.dao_address);

        SpaceEditor::remove(
            &self.neo4j,
            block,
            &account::new_id(&editor_removed.editor_address),
            &space_id,
        )
        .await?;

        Ok(())
    }

    pub async fn handle_initial_space_editors_added(
        &self,
        initial_editor_added: &geo::InitialEditorAdded,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let space_id = space::new_id(network_ids::GEO, &initial_editor_added.dao_address);

        stream::iter(&initial_editor_added.addresses)
            .map(Result::<_, HandlerError>::Ok)
            .try_for_each(|editor_address| {
                let space_id_ref = &space_id;
                async move {
                    // Create editor account and relation
                    let editor = account::new(editor_address.clone());
                    let editor_rel = SpaceEditor::new(editor.id(), space_id_ref);

                    // Insert editor account
                    editor
                        .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
                        .send()
                        .await?;

                    // Insert space editor relation
                    editor_rel
                        .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, "0")
                        .send()
                        .await?;

                    Ok(())
                }
            })
            .await?;

        tracing::info!(
            "Block #{} ({}): Added {} initial editors to space {}",
            block.block_number,
            block.timestamp,
            initial_editor_added.addresses.len(),
            space_id,
        );
        
        Ok(())
    }
}
