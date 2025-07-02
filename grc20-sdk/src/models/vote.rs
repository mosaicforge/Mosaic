//! This module contains models reserved for use by the KG Indexer.

use grc20_core::{
    ids, indexer_ids,
    mapping::{self, entity::EntityNodeRef, Relation, TriplesConversionError},
};

/// A vote cast by a user on a proposal.
///
/// `Person > VOTE_CAST > Proposal`
#[derive(Clone)]
#[grc20_core::relation]
#[grc20(relation_type = indexer_ids::VOTE_CAST_TYPE)]
pub struct VoteCast {
    #[grc20(attribute = indexer_ids::VOTE_TYPE_ATTRIBUTE)]
    pub vote_type: VoteType,
}

impl VoteCast {
    pub fn new_id(account_id: &str, proposal_id: &str) -> String {
        ids::create_id_from_unique_string(format!("VOTE:{account_id}:{proposal_id}"))
    }

    /// Creates a new vote cast with the given vote type.
    pub fn new(
        account_id: &str,
        proposal_id: &str,
        vote_type: VoteType,
    ) -> Relation<Self, EntityNodeRef> {
        Relation::new(
            Self::new_id(account_id, proposal_id),
            account_id,
            proposal_id,
            indexer_ids::VOTE_CAST_TYPE,
            "0",
            Self { vote_type },
        )
    }
}

#[derive(Clone, Debug)]
pub enum VoteType {
    Accept,
    Reject,
}

impl TryFrom<u64> for VoteType {
    type Error = String;

    fn try_from(vote: u64) -> Result<Self, Self::Error> {
        match vote {
            2 => Ok(Self::Accept),
            3 => Ok(Self::Reject),
            _ => Err(format!("Invalid vote type: {vote}")),
        }
    }
}

impl From<VoteType> for mapping::Value {
    fn from(vote_type: VoteType) -> Self {
        match vote_type {
            VoteType::Accept => mapping::Value::text("ACCEPT"),
            VoteType::Reject => mapping::Value::text("REJECT"),
        }
    }
}

impl TryFrom<mapping::Value> for VoteType {
    type Error = TriplesConversionError;

    fn try_from(value: mapping::Value) -> Result<Self, Self::Error> {
        match (value.value_type, value.value.as_str()) {
            (mapping::ValueType::Text, "ACCEPT") => Ok(Self::Accept),
            (mapping::ValueType::Text, "REJECT") => Ok(Self::Reject),
            (value_type, _) => Err(TriplesConversionError::InvalidValue(format!(
                "Invalid vote type value_type: {value_type:?}"
            ))),
        }
    }
}
