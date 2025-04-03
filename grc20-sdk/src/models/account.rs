use web3_utils::checksum_address;

use grc20_core::{ids, mapping::Entity, system_ids};

#[derive(Clone, PartialEq)]
#[grc20_core::entity]
#[grc20(schema_type = system_ids::ACCOUNT_TYPE)]
pub struct Account {
    #[grc20(attribute = system_ids::ADDRESS_ATTRIBUTE)]
    pub address: String,
}

pub fn gen_id(address: &str) -> String {
    ids::create_id_from_unique_string(checksum_address(address))
}

pub fn new(address: String) -> Entity<Account> {
    let checksummed_address = checksum_address(&address);

    Entity::new(
        gen_id(&checksummed_address),
        Account {
            address: checksummed_address,
        },
    )
    .with_type(system_ids::ACCOUNT_TYPE)
}