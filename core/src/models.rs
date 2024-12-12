//! This module contains models reserved for use by the KG Indexer.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use web3_utils::checksum_address;

use crate::{
    ids,
    pb::{self, grc20},
    system_ids,
};

pub struct BlockMetadata {
    pub cursor: String,
    pub block_number: u64,
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GeoAccount {
    pub id: String,
    pub address: String,
}

impl GeoAccount {
    pub fn new(address: String) -> Self {
        let checksummed_address = checksum_address(&address, None);
        Self {
            id: ids::create_id_from_unique_string(&checksummed_address),
            address: checksummed_address,
        }
    }

    pub fn id_from_address(address: &str) -> String {
        ids::create_id_from_unique_string(&checksum_address(address, None))
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub enum SpaceType {
    #[default]
    Public,
    Personal,
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename = "306598522df542f69ad72921c33ad84b", tag = "$type")]
pub struct Space {
    pub id: String,
    pub network: String,
    #[serde(rename = "`65da3fab6e1c48b7921a6a3260119b48`")]
    pub r#type: SpaceType,
    /// The address of the space's DAO contract.
    pub dao_contract_address: String,
    /// The address of the space plugin contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_plugin_address: Option<String>,
    /// The address of the voting plugin contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voting_plugin_address: Option<String>,
    /// The address of the member access plugin contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_access_plugin: Option<String>,
    /// The address of the personal space admin plugin contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub personal_space_admin_plugin: Option<String>,
}

/// Space editor relation.
#[derive(Deserialize, Serialize)]
pub struct SpaceEditor;

/// Space member relation.
#[derive(Deserialize, Serialize)]
pub struct SpaceMember;

/// Parent space relation (for subspaces).
#[derive(Deserialize, Serialize)]
pub struct ParentSpace;

pub struct EditProposal {
    pub name: String,
    pub proposal_id: String,
    pub space: String,
    pub space_address: String,
    pub creator: String,
    pub ops: Vec<grc20::Op>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "$type")]
pub struct Cursor {
    pub cursor: String,
    pub block_number: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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

#[derive(Deserialize, Serialize)]
pub struct VoteCast {
    pub id: String,
    pub vote_type: VoteType,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProposalType {
    AddEdit,
    ImportSpace,
    AddSubspace,
    RemoveSubspace,
    AddEditor,
    RemoveEditor,
    AddMember,
    RemoveMember,
}

impl TryFrom<pb::ipfs::ActionType> for ProposalType {
    type Error = String;

    fn try_from(action_type: pb::ipfs::ActionType) -> Result<Self, Self::Error> {
        match action_type {
            pb::ipfs::ActionType::AddMember => Ok(Self::AddMember),
            pb::ipfs::ActionType::RemoveMember => Ok(Self::RemoveMember),
            pb::ipfs::ActionType::AddEditor => Ok(Self::AddEditor),
            pb::ipfs::ActionType::RemoveEditor => Ok(Self::RemoveEditor),
            pb::ipfs::ActionType::AddSubspace => Ok(Self::AddSubspace),
            pb::ipfs::ActionType::RemoveSubspace => Ok(Self::RemoveSubspace),
            pb::ipfs::ActionType::AddEdit => Ok(Self::AddEdit),
            pb::ipfs::ActionType::ImportSpace => Ok(Self::ImportSpace),
            _ => Err(format!("Invalid action type: {:?}", action_type)),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum ProposalStatus {
    Proposed,
    Accepted,
    Rejected,
    Canceled,
    Executed,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Proposal {
    pub id: String,
    pub onchain_proposal_id: String,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub plugin_address: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Deserialize, Serialize)]
pub struct Proposals;

pub trait AsProposal {
    fn as_proposal(&self) -> &Proposal;

    fn type_id(&self) -> &'static str;
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MembershipProposalType {
    AddMember,
    RemoveMember,
}

impl TryFrom<pb::ipfs::ActionType> for MembershipProposalType {
    type Error = String;

    fn try_from(action_type: pb::ipfs::ActionType) -> Result<Self, Self::Error> {
        match action_type {
            pb::ipfs::ActionType::AddMember => Ok(Self::AddMember),
            pb::ipfs::ActionType::RemoveMember => Ok(Self::RemoveMember),
            _ => Err(format!("Invalid action type: {:?}", action_type)),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct MembershipProposal {
    #[serde(flatten)]
    pub proposal: Proposal,
    pub proposal_type: MembershipProposalType,
}

impl AsProposal for MembershipProposal {
    fn as_proposal(&self) -> &Proposal {
        &self.proposal
    }

    fn type_id(&self) -> &'static str {
        system_ids::MEMBERSHIP_PROPOSAL_TYPE
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EditorshipProposalType {
    AddEditor,
    RemoveEditor,
}

#[derive(Deserialize, Serialize)]
pub struct EditorshipProposal {
    #[serde(flatten)]
    pub proposal: Proposal,
    pub proposal_type: MembershipProposalType,
}

impl AsProposal for EditorshipProposal {
    fn as_proposal(&self) -> &Proposal {
        &self.proposal
    }

    fn type_id(&self) -> &'static str {
        system_ids::EDITORSHIP_PROPOSAL_TYPE
    }
}

#[derive(Deserialize, Serialize)]
pub struct ProposedAccount;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubspaceProposalType {
    AddSubspace,
    RemoveSubspace,
}

impl TryFrom<pb::ipfs::ActionType> for SubspaceProposalType {
    type Error = String;

    fn try_from(action_type: pb::ipfs::ActionType) -> Result<Self, Self::Error> {
        match action_type {
            pb::ipfs::ActionType::AddSubspace => Ok(Self::AddSubspace),
            pb::ipfs::ActionType::RemoveSubspace => Ok(Self::RemoveSubspace),
            _ => Err(format!("Invalid action type: {:?}", action_type)),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct SubspaceProposal {
    #[serde(flatten)]
    pub proposal: Proposal,
    pub proposal_type: SubspaceProposalType,
}

impl AsProposal for SubspaceProposal {
    fn as_proposal(&self) -> &Proposal {
        &self.proposal
    }

    fn type_id(&self) -> &'static str {
        system_ids::SUBSPACE_PROPOSAL_TYPE
    }
}

#[derive(Deserialize, Serialize)]
pub struct ProposedSubspace;
