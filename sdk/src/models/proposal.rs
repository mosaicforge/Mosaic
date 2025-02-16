use std::fmt::Display;

use serde::{Deserialize, Serialize};
use web3_utils::checksum_address;

use crate::{
    error::DatabaseError,
    ids, indexer_ids,
    mapping::{
        attributes::{FromAttributes, IntoAttributes}, entity, query_utils::{AttributeFilter, PropFilter, Query}, Entity, Relation, Value
    },
    pb::ipfs,
};

use super::BlockMetadata;

/// Common fields for all proposals
#[derive(Clone)]
pub struct Proposal {
    pub onchain_proposal_id: String,
    pub status: ProposalStatus,
    pub plugin_address: String,
    pub start_time: String,
    pub end_time: String,
}

impl Proposal {
    pub fn generate_id(proposal_id: &str) -> String {
        ids::create_id_from_unique_string(proposal_id)
    }

    /// Finds a proposal by its onchain ID and plugin address
    pub async fn find_by_id_and_address(
        neo4j: &neo4rs::Graph,
        proposal_id: &str,
        plugin_address: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        Ok(
            entity::find_many(neo4j, indexer_ids::INDEXER_SPACE_ID, None)
                .attribute(
                    AttributeFilter::new("onchain_proposal_id")
                        .value(PropFilter::new().value(checksum_address(proposal_id))),
                )
                .attribute(
                    AttributeFilter::new("plugin_address")
                        .value(PropFilter::new().value(checksum_address(plugin_address))),
                )
                .send()
                .await?
                .into_iter()
                .next(),
        )
    }

    /// Returns a query to set the status of a proposal given its ID
    pub async fn set_status(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        proposal_id: &str,
        status: ProposalStatus,
    ) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (n:`{PROPOSAL_TYPE}` {{onchain_proposal_id: $proposal_id}})
            SET n.status = $status
            SET n += {{
                `{UPDATED_AT}`: datetime($updated_at),
                `{UPDATED_AT_BLOCK}`: $updated_at_block
            }}
            "#,
            PROPOSAL_TYPE = indexer_ids::PROPOSAL_TYPE,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("proposal_id", proposal_id)
            .param("status", status.to_string())
            .param("updated_at", block.timestamp.to_rfc3339())
            .param("updated_at_block", block.block_number.to_string());

        Ok(neo4j.run(query).await?)
    }
}

impl IntoAttributes for Proposal {
    fn into_attributes(self) -> Result<crate::mapping::Attributes, crate::mapping::TriplesConversionError> {
        Ok(crate::mapping::Attributes::default()
            .attribute(("onchain_proposal_id", self.onchain_proposal_id))
            .attribute(("status", self.status.to_string()))
            .attribute(("plugin_address", self.plugin_address))
            .attribute(("start_time", self.start_time))
            .attribute(("end_time", self.end_time)))
    }
}

impl FromAttributes for Proposal {
    fn from_attributes(mut attributes: crate::mapping::Attributes) -> Result<Self, crate::mapping::TriplesConversionError> {
        Ok(Self {
            onchain_proposal_id: attributes.pop("onchain_proposal_id")?,
            status: attributes.pop("status")?,
            plugin_address: attributes.pop("plugin_address")?,
            start_time: attributes.pop("start_time")?,
            end_time: attributes.pop("end_time")?,
        })
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProposalType {
    AddEdit,
    AddSubspace,
    RemoveSubspace,
    ImportSpace,
    ArchiveSpace,
}

impl TryFrom<ipfs::ActionType> for ProposalType {
    type Error = String;

    fn try_from(action_type: ipfs::ActionType) -> Result<Self, Self::Error> {
        match action_type {
            ipfs::ActionType::AddEdit => Ok(Self::AddEdit),
            ipfs::ActionType::AddSubspace => Ok(Self::AddSubspace),
            ipfs::ActionType::RemoveSubspace => Ok(Self::RemoveSubspace),
            ipfs::ActionType::ImportSpace => Ok(Self::ImportSpace),
            ipfs::ActionType::ArchiveSpace => Ok(Self::ArchiveSpace),
            _ => Err(format!("Invalid action type: {:?}", action_type)),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProposalStatus {
    Proposed,
    Accepted,
    Rejected,
    Canceled,
    Executed,
}

impl Into<Value> for ProposalStatus {
    fn into(self) -> Value {
        match self {
            ProposalStatus::Proposed => Value::text("Proposed".to_string()),
            ProposalStatus::Accepted => Value::text("Accepted".to_string()),
            ProposalStatus::Rejected => Value::text("Rejected".to_string()),
            ProposalStatus::Canceled => Value::text("Canceled".to_string()),
            ProposalStatus::Executed => Value::text("Executed".to_string()),
        }
    }
}

impl TryFrom<Value> for ProposalStatus {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.value.as_str() {
            "Proposed" => Ok(Self::Proposed),
            "Accepted" => Ok(Self::Accepted),
            "Rejected" => Ok(Self::Rejected),
            "Canceled" => Ok(Self::Canceled),
            "Executed" => Ok(Self::Executed),
            _ => Err(format!("Invalid proposal status: {}", value.value)),
        }
    }
}

impl Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalStatus::Proposed => write!(f, "PROPOSED"),
            ProposalStatus::Accepted => write!(f, "ACCEPTED"),
            ProposalStatus::Rejected => write!(f, "REJECTED"),
            ProposalStatus::Canceled => write!(f, "CANCELED"),
            ProposalStatus::Executed => write!(f, "EXECUTED"),
        }
    }
}

// Relation for Space > PROPOSALS > Proposal
#[derive(Clone, Deserialize, Serialize)]
pub struct Proposals;

impl Proposals {
    pub fn new(space_id: &str, proposal_id: &str) -> Relation<Self> {
        Relation::new(
            &ids::create_id_from_unique_string(&format!("{space_id}:{proposal_id}")),
            space_id,
            proposal_id,
            indexer_ids::PROPOSALS,
            "0",
            Proposals {},
        )
    }
}

// Proposal > CREATOR > Account
#[derive(Clone, Deserialize, Serialize)]
pub struct ProposalCreator;

impl ProposalCreator {
    pub fn new(proposal_id: &str, account_id: &str) -> Relation<Self> {
        Relation::new(
            &ids::create_id_from_unique_string(&format!("CREATOR:{proposal_id}:{account_id}")),
            proposal_id,
            account_id,
            indexer_ids::PROPOSAL_CREATOR,
            "0",
            ProposalCreator {},
        )
    }
}

pub struct EditProposal {
    pub name: String,
    pub proposal_id: String,
    pub space: String,
    pub space_address: String,
    pub creator: String,
    pub ops: Vec<ipfs::Op>,
}

#[derive(Clone)]
pub struct AddMemberProposal {
    pub proposal: Proposal,
}

impl AddMemberProposal {
    pub fn new(proposal: Proposal) -> Entity<Self> {
        Entity::new(
            Proposal::generate_id(&proposal.onchain_proposal_id),
            Self { proposal },
        )
        .with_type(indexer_ids::PROPOSAL_TYPE)
        .with_type(indexer_ids::ADD_MEMBER_PROPOSAL)
    }
}

impl IntoAttributes for AddMemberProposal {
    fn into_attributes(self) -> Result<crate::mapping::Attributes, crate::mapping::TriplesConversionError> {
        self.proposal.into_attributes()
    }
}

impl FromAttributes for AddMemberProposal {
    fn from_attributes(attributes: crate::mapping::Attributes) -> Result<Self, crate::mapping::TriplesConversionError> {
        Ok(Self {
            proposal: Proposal::from_attributes(attributes)?,
        })
    }
}

#[derive(Clone)]
pub struct RemoveMemberProposal {
    pub proposal: Proposal,
}

impl RemoveMemberProposal {
    pub fn new(proposal: Proposal) -> Entity<Self> {
        Entity::new(
            Proposal::generate_id(&proposal.onchain_proposal_id),
            Self { proposal },
        )
        .with_type(indexer_ids::PROPOSAL_TYPE)
        .with_type(indexer_ids::REMOVE_MEMBER_PROPOSAL)
    }
}

impl IntoAttributes for RemoveMemberProposal {
    fn into_attributes(self) -> Result<crate::mapping::Attributes, crate::mapping::TriplesConversionError> {
        self.proposal.into_attributes()
    }
}

impl FromAttributes for RemoveMemberProposal {
    fn from_attributes(attributes: crate::mapping::Attributes) -> Result<Self, crate::mapping::TriplesConversionError> {
        Ok(Self {
            proposal: Proposal::from_attributes(attributes)?,
        })
    }
}

#[derive(Clone)]
pub struct AddEditorProposal {
    pub proposal: Proposal,
}

impl AddEditorProposal {
    pub fn new(proposal: Proposal) -> Entity<Self> {
        Entity::new(
            Proposal::generate_id(&proposal.onchain_proposal_id),
            Self { proposal },
        )
        .with_type(indexer_ids::PROPOSAL_TYPE)
        .with_type(indexer_ids::ADD_EDITOR_PROPOSAL)
    }
}

impl IntoAttributes for AddEditorProposal {
    fn into_attributes(self) -> Result<crate::mapping::Attributes, crate::mapping::TriplesConversionError> {
        self.proposal.into_attributes()
    }
}

impl FromAttributes for AddEditorProposal {
    fn from_attributes(attributes: crate::mapping::Attributes) -> Result<Self, crate::mapping::TriplesConversionError> {
        Ok(Self {
            proposal: Proposal::from_attributes(attributes)?,
        })
    }
}

#[derive(Clone)]
pub struct RemoveEditorProposal {
    pub proposal: Proposal,
}

impl RemoveEditorProposal {
    pub fn new(proposal: Proposal) -> Entity<Self> {
        Entity::new(
            Proposal::generate_id(&proposal.onchain_proposal_id),
            Self { proposal },
        )
        .with_type(indexer_ids::PROPOSAL_TYPE)
        .with_type(indexer_ids::REMOVE_EDITOR_PROPOSAL)
    }
}

impl IntoAttributes for RemoveEditorProposal {
    fn into_attributes(self) -> Result<crate::mapping::Attributes, crate::mapping::TriplesConversionError> {
        self.proposal.into_attributes()
    }
}

impl FromAttributes for RemoveEditorProposal {
    fn from_attributes(attributes: crate::mapping::Attributes) -> Result<Self, crate::mapping::TriplesConversionError> {
        Ok(Self {
            proposal: Proposal::from_attributes(attributes)?,
        })
    }
}

/// - AddEditorProposal > PROPOSED_ACCOUNT > ProposedAccount
/// - RemoveEditorProposal > PROPOSED_ACCOUNT > ProposedAccount
/// - AddMemberProposal > PROPOSED_ACCOUNT > ProposedAccount
/// - RemoveMemberProposal > PROPOSED_ACCOUNT > ProposedAccount
#[derive(Clone)]
pub struct ProposedAccount;

impl ProposedAccount {
    pub fn new(proposal_id: &str, account_id: &str) -> Relation<Self> {
        Relation::new(
            &ids::create_id_from_unique_string(&format!(
                "PROPOSED_ACCOUNT:{}:{}",
                proposal_id, account_id
            )),
            proposal_id,
            account_id,
            indexer_ids::PROPOSED_ACCOUNT,
            "0",
            Self {},
        )
    }
}

impl IntoAttributes for ProposedAccount {
    fn into_attributes(self) -> Result<crate::mapping::Attributes, crate::mapping::TriplesConversionError> {
        Ok(crate::mapping::Attributes::default())
    }
}

impl FromAttributes for ProposedAccount {
    fn from_attributes(_attributes: crate::mapping::Attributes) -> Result<Self, crate::mapping::TriplesConversionError> {
        Ok(Self {})
    }
}

#[derive(Clone)]
pub struct AddSubspaceProposal {
    pub proposal: Proposal,
}

impl AddSubspaceProposal {
    pub fn new(proposal: Proposal) -> Entity<Self> {
        Entity::new(
            Proposal::generate_id(&proposal.onchain_proposal_id),
            Self { proposal },
        )
        .with_type(indexer_ids::PROPOSAL_TYPE)
        .with_type(indexer_ids::ADD_SUBSPACE_PROPOSAL)
    }
}

impl IntoAttributes for AddSubspaceProposal {
    fn into_attributes(self) -> Result<crate::mapping::Attributes, crate::mapping::TriplesConversionError> {
        self.proposal.into_attributes()
    }
}

impl FromAttributes for AddSubspaceProposal {
    fn from_attributes(attributes: crate::mapping::Attributes) -> Result<Self, crate::mapping::TriplesConversionError> {
        Ok(Self {
            proposal: Proposal::from_attributes(attributes)?,
        })
    }
}


#[derive(Clone)]
pub struct RemoveSubspaceProposal {
    pub proposal: Proposal,
}

impl RemoveSubspaceProposal {
    pub fn new(proposal: Proposal) -> Entity<Self> {
        Entity::new(
            Proposal::generate_id(&proposal.onchain_proposal_id),
            Self { proposal },
        )
        .with_type(indexer_ids::PROPOSAL_TYPE)
        .with_type(indexer_ids::REMOVE_SUBSPACE_PROPOSAL)
    }
}

impl IntoAttributes for RemoveSubspaceProposal {
    fn into_attributes(self) -> Result<crate::mapping::Attributes, crate::mapping::TriplesConversionError> {
        self.proposal.into_attributes()
    }
}

impl FromAttributes for RemoveSubspaceProposal {
    fn from_attributes(attributes: crate::mapping::Attributes) -> Result<Self, crate::mapping::TriplesConversionError> {
        Ok(Self {
            proposal: Proposal::from_attributes(attributes)?,
        })
    }
}

/// AddSubspaceProposal > PROPOSED_SUBSPACE > ProposedSubspace
/// RemoveSubspaceProposal > PROPOSED_SUBSPACE > ProposedSubspace
#[derive(Clone)]
pub struct ProposedSubspace;

impl ProposedSubspace {
    pub fn new(subspace_proposal_id: &str, subspace_id: &str) -> Relation<Self> {
        Relation::new(
            &ids::create_id_from_unique_string(&format!(
                "PROPOSED_SUBSPACE:{subspace_proposal_id}:{subspace_id}",
            )),
            subspace_proposal_id,
            subspace_id,
            indexer_ids::PROPOSED_SUBSPACE,
            "0",
            Self {},
        )
    }
}

impl IntoAttributes for ProposedSubspace {
    fn into_attributes(self) -> Result<crate::mapping::Attributes, crate::mapping::TriplesConversionError> {
        Ok(crate::mapping::Attributes::default())
    }
}

impl FromAttributes for ProposedSubspace {
    fn from_attributes(_attributes: crate::mapping::Attributes) -> Result<Self, crate::mapping::TriplesConversionError> {
        Ok(Self {})
    }
}