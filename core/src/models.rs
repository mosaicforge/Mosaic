//! This module contains models reserved for use by the KG Indexer.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::pb::grc20;

#[derive(Deserialize, Serialize)]
pub enum SpaceType {
    Public,
    Personal,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "$type")]
pub struct Space {
    pub id: String,
    pub network: String,
    pub contract_address: String,
    pub dao_contract_address: String,
    pub r#type: SpaceType,
    pub created_at: DateTime<Utc>,
    pub created_at_block: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "$type")]
pub struct SpaceEditor {
    pub account_id: String,
    pub created_at: DateTime<Utc>,
    pub created_at_block: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "$type")]
pub struct SpaceMember {
    pub account_id: String,
    pub created_at: DateTime<Utc>,
    pub created_at_block: u64,
}

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
