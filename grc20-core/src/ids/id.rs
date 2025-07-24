use std::fmt::Display;

use md5::{Digest, Md5};
use uuid::{Builder, Uuid};

use super::base58::encode_uuid_to_base58;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Grc20Id(pub String);

impl Grc20Id {
    pub fn new() -> Self {
        Grc20Id(create_geo_id())
    }
}

impl Display for Grc20Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for Grc20Id {
    fn default() -> Self {
        Grc20Id::new()
    }
}

impl From<&Grc20Id> for String {
    fn from(value: &Grc20Id) -> Self {
        value.0.clone()
    }
}

impl From<Grc20Id> for String {
    fn from(value: Grc20Id) -> Self {
        value.0
    }
}

pub fn create_merged_version_id(merged_version_ids: Vec<&str>) -> Uuid {
    create_id_from_unique_string(merged_version_ids.join(","))
}

pub fn create_version_id(space_id: &str, proposal_id: &str) -> Uuid {
    create_id_from_unique_string(format!("{space_id}:{proposal_id}"))
}

pub fn create_version_id_from_block(space_id: &str, block: u64) -> Uuid {
    create_id_from_unique_string(format!("{space_id}:{block}"))
}

/**
 * A space's id is derived from the contract address of the DAO and the network the DAO is deployed to.
 * Users can import or fork a space from any network and import the contents of the original space into
 * the new one that they're creating.
 */
pub fn create_space_id(network: &str, address: &str) -> Uuid {
    create_id_from_unique_string(format!("{network}:{address}"))
}

pub fn create_id_from_unique_string(text: impl Into<String>) -> Uuid {
    let mut hasher = Md5::new();
    hasher.update(text.into());
    let hashed: [u8; 16] = hasher.finalize().into();

    Builder::from_random_bytes(hashed).into_uuid()
    // encode_uuid_to_base58(&uuid.to_string())
}

pub fn create_geo_id() -> String {
    encode_uuid_to_base58(&uuid::Uuid::new_v4().to_string())
}
