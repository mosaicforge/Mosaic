//! This module contains models reserved for use by the KG Indexer.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use web3_utils::checksum_address;

use crate::{ids, pb::grc20};

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

#[derive(Clone, Deserialize, Serialize)]
pub struct Subspace {
    pub id: String,
    pub parent_space: String,
}

/// Space editor relation.
#[derive(Deserialize, Serialize)]
pub struct SpaceEditor;

/// Space member relation.
#[derive(Deserialize, Serialize)]
pub struct SpaceMember;

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