pub mod base58;
pub mod id;
pub mod indexer_ids;
pub mod network_ids;
pub mod system_ids;

pub use id::*;

pub fn indexed(id: &str) -> bool {
    // Add other ids to this list as needed
    id == system_ids::DESCRIPTION_ATTRIBUTE || id == system_ids::NAME_ATTRIBUTE
}
