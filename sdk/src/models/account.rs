use futures::Stream;
use web3_utils::checksum_address;
use grc20_macros::entity;

use crate::{
    error::DatabaseError,
    ids, indexer_ids,
    mapping::{
        self, entity,
        entity_node::EntityFilter,
        query_utils::{AttributeFilter, PropFilter, Query, QueryStream, TypesFilter},
        Entity,
    },
    system_ids,
};

#[derive(Clone, PartialEq)]
pub struct Account {
    pub address: String,
}

impl Account {
    pub fn gen_id(address: &str) -> String {
        ids::create_id_from_unique_string(checksum_address(address))
    }

    pub fn new(address: String) -> Entity<Self> {
        let checksummed_address = checksum_address(&address);

        Entity::new(
            Self::gen_id(&checksummed_address),
            Self {
                address: checksummed_address,
            },
        )
        .with_type(system_ids::ACCOUNT_TYPE)
    }

    /// Find an account by its address
    pub fn find_one(neo4j: &neo4rs::Graph, address: &str) -> FindOneQuery {
        FindOneQuery::new(neo4j.clone(), address.to_string())
    }

    /// Find multiple accounts with filters
    pub fn find_many(neo4j: &neo4rs::Graph) -> FindManyQuery {
        FindManyQuery::new(neo4j.clone())
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

/// Query to find a single account by address
pub struct FindOneQuery {
    neo4j: neo4rs::Graph,
    address: String,
}

impl FindOneQuery {
    fn new(neo4j: neo4rs::Graph, address: String) -> Self {
        Self { neo4j, address }
    }
}

impl Query<Option<Entity<Account>>> for FindOneQuery {
    async fn send(self) -> Result<Option<Entity<Account>>, DatabaseError> {
        let account_id = Account::gen_id(&self.address);

        entity::find_one(&self.neo4j, account_id, indexer_ids::INDEXER_SPACE_ID, None)
            .send()
            .await
    }
}

/// Query to find multiple accounts with filters
pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    address: Option<PropFilter<String>>,
    limit: usize,
    skip: Option<usize>,
}

impl FindManyQuery {
    fn new(neo4j: neo4rs::Graph) -> Self {
        Self {
            neo4j,
            address: None,
            limit: 100,
            skip: None,
        }
    }

    /// Filter by account address
    pub fn address(mut self, address: PropFilter<String>) -> Self {
        self.address = Some(address);
        self
    }

    /// Limit the number of results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Skip a number of results
    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }
}

impl QueryStream<Entity<Account>> for FindManyQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Entity<Account>, DatabaseError>>, DatabaseError> {
        let mut query = entity::find_many(&self.neo4j, indexer_ids::INDEXER_SPACE_ID, None)
            .limit(self.limit)
            .with_filter(
                EntityFilter::default()
                    .relations(TypesFilter::default().r#type(system_ids::ACCOUNT_TYPE.to_string())),
            );

        if let Some(address) = self.address {
            query =
                query.attribute(AttributeFilter::new(system_ids::ADDRESS_ATTRIBUTE).value(address));
        }

        if let Some(skip) = self.skip {
            query = query.skip(skip);
        }

        query.send().await
    }
}
