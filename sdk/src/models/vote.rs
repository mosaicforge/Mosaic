//! This module contains models reserved for use by the KG Indexer.

use crate::{
    ids, indexer_ids,
    mapping::{self, Relation},
};

/// A vote cast by a user on a proposal.
///
/// `Person > VOTE_CAST > Proposal`
#[derive(Clone)]
pub struct VoteCast {
    pub vote_type: VoteType,
}

impl VoteCast {
    pub fn new_id(account_id: &str, proposal_id: &str) -> String {
        ids::create_id_from_unique_string(&format!("VOTE:{account_id}:{proposal_id}"))
    }

    /// Creates a new vote cast with the given vote type.
    pub fn new(account_id: &str, proposal_id: &str, vote_type: VoteType) -> Relation<Self> {
        Relation::new(
            &Self::new_id(account_id, proposal_id),
            account_id,
            proposal_id,
            indexer_ids::VOTE_CAST_TYPE,
            "0",
            Self { vote_type },
        )
    }
}

impl mapping::IntoAttributes for VoteCast {
    fn into_attributes(self) -> Result<mapping::Attributes, mapping::TriplesConversionError> {
        Ok(mapping::Attributes::default()
            .attribute((indexer_ids::VOTE_TYPE_ATTRIBUTE, self.vote_type)))
    }
}

impl mapping::FromAttributes for VoteCast {
    fn from_attributes(
        mut attributes: mapping::Attributes,
    ) -> Result<Self, mapping::TriplesConversionError> {
        Ok(Self {
            vote_type: attributes.pop(indexer_ids::VOTE_TYPE_ATTRIBUTE)?,
        })
    }
}

#[derive(Clone)]
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
            _ => Err(format!("Invalid vote type: {}", vote)),
        }
    }
}

impl Into<mapping::Value> for VoteType {
    fn into(self) -> mapping::Value {
        match self {
            Self::Accept => mapping::Value::text("ACCEPT"),
            Self::Reject => mapping::Value::text("REJECT"),
        }
    }
}

impl TryFrom<mapping::Value> for VoteType {
    type Error = String;

    fn try_from(value: mapping::Value) -> Result<Self, Self::Error> {
        match (value.value_type, value.value.as_str()) {
            (mapping::ValueType::Text, "ACCEPT") => Ok(Self::Accept),
            (mapping::ValueType::Text, "REJECT") => Ok(Self::Reject),
            (value_type, _) => Err(format!("Invalid vote type value_type: {:?}", value_type)),
        }
    }
}
