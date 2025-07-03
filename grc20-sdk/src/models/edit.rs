use grc20_core::{
    ids, indexer_ids,
    mapping::{entity::EntityNodeRef, Entity, Relation},
    system_ids,
};

#[grc20_core::entity]
#[grc20(schema_type = indexer_ids::EDIT_TYPE)]
pub struct Edit {
    #[grc20(attribute = system_ids::NAME_ATTRIBUTE)]
    pub name: String,
    #[grc20(attribute = indexer_ids::EDIT_CONTENT_URI_ATTRIBUTE)]
    pub content_uri: String,
    #[grc20(attribute = indexer_ids::EDIT_INDEX_ATTRIBUTE)]
    pub index: Option<String>,
}

impl Edit {
    pub fn gen_id(content_uri: &str) -> String {
        ids::create_id_from_unique_string(content_uri)
    }

    pub fn new(name: String, content_uri: String, index: Option<String>) -> Entity<Self> {
        Entity::new(
            Self::gen_id(&content_uri),
            Self {
                name,
                content_uri,
                index,
            },
        )
        .with_type(indexer_ids::EDIT_TYPE)
    }
}

/// Space > EDITS > Edit
#[derive(Clone)]
#[grc20_core::relation]
#[grc20(relation_type = indexer_ids::EDITS)]
pub struct Edits;

impl Edits {
    pub fn gen_id(space_id: &str, edit_id: &str) -> String {
        ids::create_id_from_unique_string(format!("{space_id}:{edit_id}"))
    }

    pub fn new(
        space_id: impl Into<String>,
        edit_id: impl Into<String>,
    ) -> Relation<Self, EntityNodeRef> {
        let space_id = space_id.into();
        let edit_id = edit_id.into();

        Relation::new(
            Self::gen_id(&space_id, &edit_id),
            space_id,
            edit_id,
            indexer_ids::EDITS,
            "0",
            Self {},
        )
    }
}

/// EditProposal > PROPOSED_EDIT > Edit
#[derive(Clone)]
#[grc20_core::relation]
#[grc20(relation_type = indexer_ids::PROPOSED_EDIT)]
pub struct ProposedEdit;

impl ProposedEdit {
    pub fn gen_id(proposal_id: &str, edit_id: &str) -> String {
        ids::create_id_from_unique_string(format!("{proposal_id}:{edit_id}"))
    }

    pub fn new(
        proposal_id: impl Into<String>,
        edit_id: impl Into<String>,
    ) -> Relation<Self, EntityNodeRef> {
        let proposal_id = proposal_id.into();
        let edit_id = edit_id.into();

        Relation::new(
            Self::gen_id(&proposal_id, &edit_id),
            proposal_id,
            edit_id,
            indexer_ids::PROPOSED_EDIT,
            "0",
            Self {},
        )
    }
}
