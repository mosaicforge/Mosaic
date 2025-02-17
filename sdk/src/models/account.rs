use web3_utils::checksum_address;

use crate::{
    ids,
    mapping::{self, Entity},
    system_ids,
};

#[derive(Clone, PartialEq)]
pub struct Account {
    pub address: String,
}

impl Account {
    pub fn generate_id(address: &str) -> String {
        ids::create_id_from_unique_string(&checksum_address(address))
    }

    pub fn new(address: String) -> Entity<Self> {
        let checksummed_address = checksum_address(&address);

        Entity::new(
            Self::generate_id(&checksummed_address),
            Self {
                address: checksummed_address,
            },
        )
        .with_type(system_ids::ACCOUNT_TYPE)
    }
}

impl mapping::IntoAttributes for Account {
    fn into_attributes(self) -> Result<mapping::Attributes, mapping::TriplesConversionError> {
        Ok(mapping::Attributes::default().attribute((system_ids::ADDRESS_ATTRIBUTE, self.address)))
    }
}

impl mapping::FromAttributes for Account {
    fn from_attributes(
        mut attributes: mapping::Attributes,
    ) -> Result<Self, mapping::TriplesConversionError> {
        Ok(Self {
            address: attributes.pop(system_ids::ADDRESS_ATTRIBUTE)?,
        })
    }
}
