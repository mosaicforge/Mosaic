use futures::try_join;
use sdk::{
    indexer_ids, mapping::query_utils::Query, models::{Account, BlockMetadata, Space, SpaceEditor}, pb::geo
};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_editor_added(
        &self,
        editor_added: &geo::EditorAdded,
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        // match try_join!(
        //     Space::find_by_voting_plugin_address(
        //         &self.neo4j,
        //         &editor_added.main_voting_plugin_address,
        //     ),
        //     Space::find_by_personal_plugin_address(
        //         &self.neo4j,
        //         &editor_added.main_voting_plugin_address
        //     )
        // )? {
        //     // Space found
        //     (Some(space), _) | (None, Some(space)) => {
        //         // Create editor account and space editor relation
        //         let editor = Account::new(editor_added.editor_address.clone());
        //         let editor_relation = SpaceEditor::new(&editor.id, &space.id);

        //         // Insert editor account
        //         editor.insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, 0)
        //             .send()
        //             .await?;

        //         // Insert space editor relation
        //         editor_relation
        //             .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, 0)
        //             .send()
        //             .await?;
        //     }
        //     // Space not found
        //     (None, None) => {
        //         tracing::warn!(
        //             "Block #{} ({}): Could not add editor for unknown space with voting_plugin_address = {}",
        //             block.block_number,
        //             block.timestamp,
        //             checksum_address(&editor_added.main_voting_plugin_address)
        //         );
        //     }
        // }

        if let Some(space) = Space::find_by_dao_address(&self.neo4j, &editor_added.dao_address)
            .await?
        {
            // Create editor account and space editor relation
            let editor = Account::new(editor_added.editor_address.clone());
            let editor_relation = SpaceEditor::new(&editor.id, &space.id);

            // Insert editor account
            editor.insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, 0)
                .send()
                .await?;

            // Insert space editor relation
            editor_relation
                .insert(&self.neo4j, block, indexer_ids::INDEXER_SPACE_ID, 0)
                .send()
                .await?;
        } else {
            tracing::warn!(
                "Block #{} ({}): Could not add editor for unknown space with dao_address = {}",
                block.block_number,
                block.timestamp,
                checksum_address(&editor_added.dao_address)
            );
        }

        Ok(())
    }
}
