use crate::{ids, indexer_ids, mapping::{Attributes, Entity, FromAttributes, IntoAttributes, Relation, TriplesConversionError}, system_ids};

pub struct Edit {
    pub name: String,
    pub content_uri: String,
    pub index: Option<String>,
}

impl Edit {
    pub fn gen_id(content_uri: &str) -> String {
        ids::create_id_from_unique_string(content_uri)
    }

    pub fn new(
        name: String,
        content_uri: String,
        index: Option<String>
    ) -> Entity<Self> {
        Entity::new(Self::gen_id(&content_uri), Self { name, content_uri, index })
            .with_type(indexer_ids::EDIT_TYPE)
    }
}

impl IntoAttributes for Edit {
    fn into_attributes(self) -> Result<Attributes, TriplesConversionError> {
        let attrs = Attributes::default()
            .attribute((system_ids::NAME_ATTRIBUTE, self.name))
            .attribute((indexer_ids::EDIT_CONTENT_URI_ATTRIBUTE, self.content_uri));

        if let Some(index) = self.index {
            Ok(attrs.attribute((indexer_ids::EDIT_INDEX_ATTRIBUTE, index)))
        } else {
            Ok(attrs)
        }
    }
}

impl FromAttributes for Edit {
    fn from_attributes(attributes: Attributes) -> Result<Self, TriplesConversionError> {
        Ok(Self {
            name: attributes.get(system_ids::NAME_ATTRIBUTE)?,
            content_uri: attributes.get(indexer_ids::EDIT_CONTENT_URI_ATTRIBUTE)?,
            index: attributes.get_opt(indexer_ids::EDIT_INDEX_ATTRIBUTE)?,
        })
    }
}

/// Space > EDITS > Edit
#[derive(Debug, Clone)]
pub struct Edits;

impl Edits {
    pub fn gen_id(space_id: &str, edit_id: &str) -> String {
        ids::create_id_from_unique_string(&format!("{}:{}", space_id, edit_id))
    }

    pub fn new(
        space_id: impl Into<String>,
        edit_id: impl Into<String>,
    ) -> Relation<Self> {
        let space_id = space_id.into();
        let edit_id = edit_id.into();

        Relation::new(
            Self::gen_id(&space_id, &edit_id),
            space_id,
            edit_id,
            indexer_ids::EDITS,
            "0",
            Self {}
        )
    }
}

impl FromAttributes for Edits {
    fn from_attributes(_attributes: Attributes) -> Result<Self, TriplesConversionError> {
        Ok(Self {})
    }
}

impl IntoAttributes for Edits {
    fn into_attributes(self) -> Result<Attributes, TriplesConversionError> {
        Ok(Attributes::default())
    }
}

/// EditProposal > PROPOSED_EDIT > Edit
#[derive(Debug, Clone)]
pub struct ProposedEdit;

impl ProposedEdit {
    pub fn gen_id(proposal_id: &str, edit_id: &str) -> String {
        ids::create_id_from_unique_string(&format!("{}:{}", proposal_id, edit_id))
    }

    pub fn new(
        proposal_id: impl Into<String>,
        edit_id: impl Into<String>,
    ) -> Relation<Self> {
        let proposal_id = proposal_id.into();
        let edit_id = edit_id.into();

        Relation::new(
            Self::gen_id(&proposal_id, &edit_id),
            proposal_id,
            edit_id,
            indexer_ids::PROPOSED_EDIT,
            "0",
            Self {}
        )
    }
}

impl FromAttributes for ProposedEdit {
    fn from_attributes(_attributes: Attributes) -> Result<Self, TriplesConversionError> {
        Ok(Self {})
    }
}

impl IntoAttributes for ProposedEdit {
    fn into_attributes(self) -> Result<Attributes, TriplesConversionError> {
        Ok(Attributes::default())
    }
}