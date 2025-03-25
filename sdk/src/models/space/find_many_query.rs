use futures::Stream;

use crate::{error::DatabaseError, indexer_ids, mapping::{entity, entity_node::EntityFilter, query_utils::{QueryStream, TypesFilter}, AttributeFilter, Entity, PropFilter}, system_ids};

use super::Space;

/// Query to find multiple spaces with filters
pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    network: Option<PropFilter<String>>,
    governance_type: Option<PropFilter<String>>,
    dao_contract_address: Option<PropFilter<String>>,
    space_plugin_address: Option<PropFilter<String>>,
    voting_plugin_address: Option<PropFilter<String>>,
    member_access_plugin: Option<PropFilter<String>>,
    personal_space_admin_plugin: Option<PropFilter<String>>,
    limit: usize,
    skip: Option<usize>,
}

impl FindManyQuery {
    pub(crate) fn new(neo4j: neo4rs::Graph) -> Self {
        Self {
            neo4j,
            network: None,
            governance_type: None,
            dao_contract_address: None,
            space_plugin_address: None,
            voting_plugin_address: None,
            member_access_plugin: None,
            personal_space_admin_plugin: None,
            limit: 100,
            skip: None,
        }
    }

    /// Filter by network
    pub fn network(mut self, network: PropFilter<String>) -> Self {
        self.network = Some(network);
        self
    }

    /// Filter by governance type
    pub fn governance_type(mut self, governance_type: PropFilter<String>) -> Self {
        self.governance_type = Some(governance_type);
        self
    }

    /// Filter by DAO contract address
    pub fn dao_contract_address(mut self, dao_contract_address: PropFilter<String>) -> Self {
        self.dao_contract_address = Some(dao_contract_address);
        self
    }

    /// Filter by space plugin address
    pub fn space_plugin_address(mut self, space_plugin_address: PropFilter<String>) -> Self {
        self.space_plugin_address = Some(space_plugin_address);
        self
    }

    /// Filter by voting plugin address
    pub fn voting_plugin_address(mut self, voting_plugin_address: PropFilter<String>) -> Self {
        self.voting_plugin_address = Some(voting_plugin_address);
        self
    }

    /// Filter by member access plugin
    pub fn member_access_plugin(mut self, member_access_plugin: PropFilter<String>) -> Self {
        self.member_access_plugin = Some(member_access_plugin);
        self
    }

    /// Filter by personal space admin plugin
    pub fn personal_space_admin_plugin(
        mut self,
        personal_space_admin_plugin: PropFilter<String>,
    ) -> Self {
        self.personal_space_admin_plugin = Some(personal_space_admin_plugin);
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

impl QueryStream<Entity<Space>> for FindManyQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Entity<Space>, DatabaseError>>, DatabaseError> {
        let mut query = entity::find_many(&self.neo4j, indexer_ids::INDEXER_SPACE_ID, None)
            .limit(self.limit)
            .with_filter(
                EntityFilter::default()
                    .relations(TypesFilter::default().r#type(system_ids::SPACE_TYPE.to_string())),
            );

        if let Some(network) = self.network {
            query =
                query.attribute(AttributeFilter::new(system_ids::NETWORK_ATTRIBUTE).value(network));
        }

        if let Some(governance_type) = self.governance_type {
            query = query.attribute(
                AttributeFilter::new(indexer_ids::SPACE_GOVERNANCE_TYPE).value(governance_type),
            );
        }

        if let Some(dao_contract_address) = self.dao_contract_address {
            query = query.attribute(
                AttributeFilter::new(indexer_ids::SPACE_DAO_ADDRESS).value(dao_contract_address),
            );
        }

        if let Some(space_plugin_address) = self.space_plugin_address {
            query = query.attribute(
                AttributeFilter::new(indexer_ids::SPACE_PLUGIN_ADDRESS).value(space_plugin_address),
            );
        }

        if let Some(voting_plugin_address) = self.voting_plugin_address {
            query = query.attribute(
                AttributeFilter::new(indexer_ids::SPACE_VOTING_PLUGIN_ADDRESS)
                    .value(voting_plugin_address),
            );
        }

        if let Some(member_access_plugin) = self.member_access_plugin {
            query = query.attribute(
                AttributeFilter::new(indexer_ids::SPACE_MEMBER_PLUGIN_ADDRESS)
                    .value(member_access_plugin),
            );
        }

        if let Some(personal_space_admin_plugin) = self.personal_space_admin_plugin {
            query = query.attribute(
                AttributeFilter::new(indexer_ids::SPACE_PERSONAL_PLUGIN_ADDRESS)
                    .value(personal_space_admin_plugin),
            );
        }

        if let Some(skip) = self.skip {
            query = query.skip(skip);
        }

        query.send().await
    }
}
