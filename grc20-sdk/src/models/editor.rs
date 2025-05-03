use grc20_core::{
    block::BlockMetadata,
    error::DatabaseError,
    ids, indexer_ids,
    mapping::{entity::EntityNodeRef, query_utils::Query, relation, Relation},
    neo4rs,
};

/// Space editor relation.
/// Account > EDITOR > Space
#[derive(Clone)]
#[grc20_core::relation]
#[grc20(relation_type = indexer_ids::EDITOR_RELATION)]
pub struct SpaceEditor;

impl SpaceEditor {
    pub fn generate_id(editor_id: &str, space_id: &str) -> String {
        ids::create_id_from_unique_string(format!("EDITOR:{space_id}:{editor_id}"))
    }

    pub fn new(editor_id: &str, space_id: &str) -> Relation<Self, EntityNodeRef> {
        Relation::new(
            Self::generate_id(editor_id, space_id),
            editor_id,
            space_id,
            indexer_ids::EDITOR_RELATION,
            "0",
            Self,
        )
    }

    /// Delete a relation between an editor and a space.
    pub async fn remove(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        editor_id: &str,
        space_id: &str,
    ) -> Result<(), DatabaseError> {
        relation::delete_one(
            neo4j,
            block,
            SpaceEditor::generate_id(editor_id, space_id),
            indexer_ids::INDEXER_SPACE_ID,
            "0",
        )
        .send()
        .await
    }
}
