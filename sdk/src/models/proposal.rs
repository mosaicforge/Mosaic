use std::fmt::Display;

use serde::{Deserialize, Serialize};
use web3_utils::checksum_address;

use crate::{
    error::DatabaseError,
    ids, indexer_ids,
    mapping::{
        entity,
        query_utils::{AttributeFilter, PropFilter, Query},
        Entity, Relation,
    },
    pb::ipfs,
};

use super::BlockMetadata;

/// Common fields for all proposals
#[derive(Clone, Deserialize, Serialize)]
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

#[derive(Clone, Deserialize, Serialize)]
pub struct AddMemberProposal {
    #[serde(flatten)]
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

#[derive(Clone, Deserialize, Serialize)]
pub struct RemoveMemberProposal {
    #[serde(flatten)]
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

#[derive(Clone, Deserialize, Serialize)]
pub struct AddEditorProposal {
    #[serde(flatten)]
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

#[derive(Clone, Deserialize, Serialize)]
pub struct RemoveEditorProposal {
    #[serde(flatten)]
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

/// - AddEditorProposal > PROPOSED_ACCOUNT > ProposedAccount
/// - RemoveEditorProposal > PROPOSED_ACCOUNT > ProposedAccount
/// - AddMemberProposal > PROPOSED_ACCOUNT > ProposedAccount
/// - RemoveMemberProposal > PROPOSED_ACCOUNT > ProposedAccount
#[derive(Clone, Deserialize, Serialize)]
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

#[derive(Clone, Deserialize, Serialize)]
pub struct AddSubspaceProposal {
    #[serde(flatten)]
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

#[derive(Clone, Deserialize, Serialize)]
pub struct RemoveSubspaceProposal {
    #[serde(flatten)]
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

/// AddSubspaceProposal > PROPOSED_SUBSPACE > ProposedSubspace
/// RemoveSubspaceProposal > PROPOSED_SUBSPACE > ProposedSubspace
#[derive(Clone, Deserialize, Serialize)]
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
