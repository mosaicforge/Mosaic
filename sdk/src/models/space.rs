use futures::{pin_mut, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use web3_utils::checksum_address;

use crate::{
    error::DatabaseError,
    ids, indexer_ids,
    mapping::{
        self,
        attributes::{FromAttributes, IntoAttributes},
        entity, entity_node::{self, EntityFilter}, prop_filter,
        query_utils::{AttributeFilter, PropFilter, Query, QueryStream, TypesFilter},
        relation, relation_node, Attributes, Entity, EntityNode, Relation, TriplesConversionError,
        Value,
    },
    models::Account,
    network_ids, system_ids,
};

use super::BlockMetadata;

#[derive(Clone)]
pub struct Space {
    pub network: String,
    pub governance_type: SpaceGovernanceType,
    /// The address of the space's DAO contract.
    pub dao_contract_address: String,
    /// The address of the space plugin contract.
    pub space_plugin_address: Option<String>,
    /// The address of the voting plugin contract.
    pub voting_plugin_address: Option<String>,
    /// The address of the member access plugin contract.
    pub member_access_plugin: Option<String>,
    /// The address of the personal space admin plugin contract.
    pub personal_space_admin_plugin: Option<String>,
}

impl Space {
    pub fn gen_id(network: &str, address: &str) -> String {
        ids::create_id_from_unique_string(format!("{network}:{}", checksum_address(address)))
    }

    pub fn builder(id: &str, dao_contract_address: &str) -> SpaceBuilder {
        SpaceBuilder::new(id, dao_contract_address)
    }

    /// Find a space by its DAO contract address.
    pub async fn find_by_dao_address(
        neo4j: &neo4rs::Graph,
        dao_contract_address: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        entity::find_one(
            neo4j,
            Space::gen_id(network_ids::GEO, dao_contract_address),
            indexer_ids::INDEXER_SPACE_ID,
            None,
        )
        .send()
        .await
    }

    pub async fn find_entity_by_dao_address(
        neo4j: &neo4rs::Graph,
        dao_contract_address: &str,
    ) -> Result<Option<EntityNode>, DatabaseError> {
        entity_node::find_one(neo4j, Space::gen_id(network_ids::GEO, dao_contract_address))
            .send()
            .await
    }

    /// Find a space by its space plugin address.
    pub async fn find_by_space_plugin_address(
        neo4j: &neo4rs::Graph,
        space_plugin_address: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        let stream = entity::find_many(neo4j, indexer_ids::INDEXER_SPACE_ID, None)
            .attribute(
                AttributeFilter::new(indexer_ids::SPACE_PLUGIN_ADDRESS)
                    .value(PropFilter::default().value(checksum_address(space_plugin_address))),
            )
            .limit(1)
            .send()
            .await?;

        pin_mut!(stream);

        stream.next().await.transpose()
    }

    pub async fn find_entity_by_space_plugin_address(
        neo4j: &neo4rs::Graph,
        space_plugin_address: &str,
    ) -> Result<Option<EntityNode>, DatabaseError> {
        let stream = entity_node::find_many(neo4j)
            .attribute(
                AttributeFilter::new(indexer_ids::SPACE_PLUGIN_ADDRESS)
                    .space_id(prop_filter::value(indexer_ids::INDEXER_SPACE_ID))
                    .value(prop_filter::value(checksum_address(space_plugin_address))),
            )
            .limit(1)
            .send()
            .await?;

        pin_mut!(stream);

        stream.next().await.transpose()
    }

    /// Find a space by its voting plugin address.
    pub async fn find_by_voting_plugin_address(
        neo4j: &neo4rs::Graph,
        voting_plugin_address: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        let stream = entity::find_many(neo4j, indexer_ids::INDEXER_SPACE_ID, None)
            .attribute(
                AttributeFilter::new(indexer_ids::SPACE_VOTING_PLUGIN_ADDRESS)
                    .value(PropFilter::default().value(checksum_address(voting_plugin_address))),
            )
            .limit(1)
            .send()
            .await?;

        pin_mut!(stream);

        stream.next().await.transpose()
    }

    pub async fn find_entity_by_voting_plugin_address(
        neo4j: &neo4rs::Graph,
        voting_plugin_address: &str,
    ) -> Result<Option<EntityNode>, DatabaseError> {
        let stream = entity_node::find_many(neo4j)
            .attribute(
                AttributeFilter::new(indexer_ids::SPACE_VOTING_PLUGIN_ADDRESS)
                    .space_id(prop_filter::value(indexer_ids::INDEXER_SPACE_ID))
                    .value(prop_filter::value(checksum_address(voting_plugin_address))),
            )
            .limit(1)
            .send()
            .await?;

        pin_mut!(stream);

        stream.next().await.transpose()
    }

    /// Find a space by its member access plugin address.
    pub async fn find_by_member_access_plugin(
        neo4j: &neo4rs::Graph,
        member_access_plugin: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        let stream = entity::find_many(neo4j, indexer_ids::INDEXER_SPACE_ID, None)
            .attribute(
                AttributeFilter::new(indexer_ids::SPACE_MEMBER_PLUGIN_ADDRESS)
                    .value(PropFilter::default().value(checksum_address(member_access_plugin))),
            )
            .limit(1)
            .send()
            .await?;

        pin_mut!(stream);

        stream.next().await.transpose()
    }

    /// Find a space by its personal space admin plugin address.
    pub async fn find_by_personal_plugin_address(
        neo4j: &neo4rs::Graph,
        personal_space_admin_plugin: &str,
    ) -> Result<Option<Entity<Self>>, DatabaseError> {
        let stream = entity::find_many(neo4j, indexer_ids::INDEXER_SPACE_ID, None)
            .attribute(
                AttributeFilter::new(indexer_ids::SPACE_PERSONAL_PLUGIN_ADDRESS).value(
                    PropFilter::default().value(checksum_address(personal_space_admin_plugin)),
                ),
            )
            .limit(1)
            .send()
            .await?;

        pin_mut!(stream);

        stream.next().await.transpose()
    }

    /// Find a space by its ID
    pub fn find_one(neo4j: &neo4rs::Graph, space_id: &str) -> FindOneQuery {
        FindOneQuery::new(neo4j.clone(), space_id.to_string())
    }

    /// Find multiple spaces with filters
    pub fn find_many(neo4j: &neo4rs::Graph) -> FindManyQuery {
        FindManyQuery::new(neo4j.clone())
    }

    /// Find all members of a space
    pub fn members(neo4j: &neo4rs::Graph, space_id: &str) -> SpaceMembersQuery {
        SpaceMembersQuery::new(neo4j.clone(), space_id.to_string())
    }

    /// Find all editors of a space
    pub fn editors(neo4j: &neo4rs::Graph, space_id: &str) -> SpaceEditorsQuery {
        SpaceEditorsQuery::new(neo4j.clone(), space_id.to_string())
    }

    /// Find all parent spaces of a given space
    pub fn parent_spaces(neo4j: &neo4rs::Graph, space_id: &str) -> ParentSpacesQuery {
        ParentSpacesQuery::new(neo4j.clone(), space_id.to_string())
    }

    /// Find all subspaces of a given space
    pub fn subspaces(neo4j: &neo4rs::Graph, space_id: &str) -> SubspacesQuery {
        SubspacesQuery::new(neo4j.clone(), space_id.to_string())
    }

    /// Find all types defined in a space
    pub fn types(neo4j: &neo4rs::Graph, space_id: &str) -> SpaceTypesQuery {
        SpaceTypesQuery::new(neo4j.clone(), space_id.to_string())
    }
}

/// Query to find all types defined in a space
pub struct SpaceTypesQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
    limit: usize,
    skip: Option<usize>,
}

impl SpaceTypesQuery {
    fn new(neo4j: neo4rs::Graph, space_id: String) -> Self {
        Self {
            neo4j,
            space_id,
            limit: 100,
            skip: None,
        }
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

impl QueryStream<EntityNode> for SpaceTypesQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<EntityNode, DatabaseError>>, DatabaseError> {
        // Find all entities that have a TYPES relation to the Type entity
        let mut stream = entity_node::find_many(&self.neo4j)
            .with_filter(
                EntityFilter::default()
                    .relations(TypesFilter::default().r#type(system_ids::SCHEMA_TYPE))
                    .space_id(self.space_id),
            )
            .limit(self.limit);

        if let Some(skip) = self.skip {
            stream = stream.skip(skip);
        }

        stream.send().await
    }
}

/// Query to find a single space by ID
pub struct FindOneQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
}

impl FindOneQuery {
    fn new(neo4j: neo4rs::Graph, space_id: String) -> Self {
        Self { neo4j, space_id }
    }
}

impl Query<Option<Entity<Space>>> for FindOneQuery {
    async fn send(self) -> Result<Option<Entity<Space>>, DatabaseError> {
        entity::find_one(
            &self.neo4j,
            self.space_id,
            indexer_ids::INDEXER_SPACE_ID,
            None,
        )
        .send()
        .await
    }
}

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
    fn new(neo4j: neo4rs::Graph) -> Self {
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
        let mut query =
            entity::find_many(&self.neo4j, indexer_ids::INDEXER_SPACE_ID, None)
                .limit(self.limit)
                .with_filter(EntityFilter::default().relations(TypesFilter::default().r#type(
                    system_ids::SPACE_TYPE.to_string(),
                )));

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

/// Query to find all members of a space
pub struct SpaceMembersQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
    limit: usize,
    skip: Option<usize>,
}

impl SpaceMembersQuery {
    fn new(neo4j: neo4rs::Graph, space_id: String) -> Self {
        Self {
            neo4j,
            space_id,
            limit: 100,
            skip: None,
        }
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

impl QueryStream<Entity<Account>> for SpaceMembersQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Entity<Account>, DatabaseError>>, DatabaseError> {
        // Find all member relations for the space
        let relations_stream = relation_node::find_many(&self.neo4j)
            .relation_type(PropFilter::default().value(indexer_ids::MEMBER_RELATION))
            .to_id(PropFilter::default().value(self.space_id.clone()))
            .space_id(PropFilter::default().value(indexer_ids::INDEXER_SPACE_ID))
            .limit(self.limit)
            .send()
            .await?;

        // Convert the stream of relations to a stream of accounts
        let neo4j = self.neo4j.clone();
        let account_stream = relations_stream
            .map(move |relation_result| {
                let neo4j = neo4j.clone();
                async move {
                    let relation = relation_result?;
                    entity::find_one(&neo4j, &relation.from, indexer_ids::INDEXER_SPACE_ID, None)
                        .send()
                        .await?
                        .ok_or_else(|| {
                            DatabaseError::NotFound(format!(
                                "Account with ID {} not found",
                                relation.from
                            ))
                        })
                }
            })
            .buffered(10); // Process up to 10 accounts concurrently

        Ok(account_stream)
    }
}

/// Query to find all editors of a space
pub struct SpaceEditorsQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
    limit: usize,
    skip: Option<usize>,
}

impl SpaceEditorsQuery {
    fn new(neo4j: neo4rs::Graph, space_id: String) -> Self {
        Self {
            neo4j,
            space_id,
            limit: 100,
            skip: None,
        }
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

impl QueryStream<Entity<Account>> for SpaceEditorsQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Entity<Account>, DatabaseError>>, DatabaseError> {
        // Find all editor relations for the space
        let relations_stream = relation_node::find_many(&self.neo4j)
            .relation_type(PropFilter::default().value(indexer_ids::EDITOR_RELATION))
            .to_id(PropFilter::default().value(self.space_id.clone()))
            .space_id(PropFilter::default().value(indexer_ids::INDEXER_SPACE_ID))
            .limit(self.limit)
            .send()
            .await?;

        // Convert the stream of relations to a stream of accounts
        let neo4j = self.neo4j.clone();
        let account_stream = relations_stream
            .map(move |relation_result| {
                let neo4j = neo4j.clone();
                async move {
                    let relation = relation_result?;
                    entity::find_one(&neo4j, &relation.from, indexer_ids::INDEXER_SPACE_ID, None)
                        .send()
                        .await?
                        .ok_or_else(|| {
                            DatabaseError::NotFound(format!(
                                "Account with ID {} not found",
                                relation.from
                            ))
                        })
                }
            })
            .buffered(10); // Process up to 10 accounts concurrently

        Ok(account_stream)
    }
}

/// Query to find all parent spaces of a given space
pub struct ParentSpacesQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
    limit: usize,
    skip: Option<usize>,
}

impl ParentSpacesQuery {
    fn new(neo4j: neo4rs::Graph, space_id: String) -> Self {
        Self {
            neo4j,
            space_id,
            limit: 100,
            skip: None,
        }
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

impl QueryStream<Entity<Space>> for ParentSpacesQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Entity<Space>, DatabaseError>>, DatabaseError> {
        // Find all parent space relations for the space
        let relations_stream = relation_node::find_many(&self.neo4j)
            .relation_type(PropFilter::default().value(indexer_ids::PARENT_SPACE))
            .from_id(PropFilter::default().value(self.space_id.clone()))
            .space_id(PropFilter::default().value(indexer_ids::INDEXER_SPACE_ID))
            .limit(self.limit)
            .send()
            .await?;

        // Convert the stream of relations to a stream of spaces
        let neo4j = self.neo4j.clone();
        let space_stream = relations_stream
            .map(move |relation_result| {
                let neo4j = neo4j.clone();
                async move {
                    let relation = relation_result?;
                    entity::find_one(&neo4j, &relation.to, indexer_ids::INDEXER_SPACE_ID, None)
                        .send()
                        .await?
                        .ok_or_else(|| {
                            DatabaseError::NotFound(format!(
                                "Space with ID {} not found",
                                relation.to
                            ))
                        })
                }
            })
            .buffered(10); // Process up to 10 spaces concurrently

        Ok(space_stream)
    }
}

/// Query to find all subspaces of a given space
pub struct SubspacesQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
    limit: usize,
    skip: Option<usize>,
}

impl SubspacesQuery {
    fn new(neo4j: neo4rs::Graph, space_id: String) -> Self {
        Self {
            neo4j,
            space_id,
            limit: 100,
            skip: None,
        }
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

impl QueryStream<Entity<Space>> for SubspacesQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<Entity<Space>, DatabaseError>>, DatabaseError> {
        // Find all parent space relations where this space is the parent
        let relations_stream = relation_node::find_many(&self.neo4j)
            .relation_type(PropFilter::default().value(indexer_ids::PARENT_SPACE))
            .to_id(PropFilter::default().value(self.space_id.clone()))
            .space_id(PropFilter::default().value(indexer_ids::INDEXER_SPACE_ID))
            .limit(self.limit)
            .send()
            .await?;

        // Convert the stream of relations to a stream of spaces
        let neo4j = self.neo4j.clone();
        let space_stream = relations_stream
            .map(move |relation_result| {
                let neo4j = neo4j.clone();
                async move {
                    let relation = relation_result?;
                    entity::find_one(&neo4j, &relation.from, indexer_ids::INDEXER_SPACE_ID, None)
                        .send()
                        .await?
                        .ok_or_else(|| {
                            DatabaseError::NotFound(format!(
                                "Space with ID {} not found",
                                relation.from
                            ))
                        })
                }
            })
            .buffered(10); // Process up to 10 spaces concurrently

        Ok(space_stream)
    }
}

impl IntoAttributes for Space {
    fn into_attributes(self) -> Result<Attributes, TriplesConversionError> {
        let mut attributes = Attributes::default()
            .attribute((system_ids::NETWORK_ATTRIBUTE, self.network))
            .attribute((indexer_ids::SPACE_GOVERNANCE_TYPE, self.governance_type))
            .attribute((indexer_ids::SPACE_DAO_ADDRESS, self.dao_contract_address));

        if let Some(space_plugin_address) = self.space_plugin_address {
            attributes.attribute_mut((indexer_ids::SPACE_PLUGIN_ADDRESS, space_plugin_address))
        }

        if let Some(voting_plugin_address) = self.voting_plugin_address {
            attributes.attribute_mut((
                indexer_ids::SPACE_VOTING_PLUGIN_ADDRESS,
                voting_plugin_address,
            ))
        }

        if let Some(member_access_plugin) = self.member_access_plugin {
            attributes.attribute_mut((
                indexer_ids::SPACE_MEMBER_PLUGIN_ADDRESS,
                member_access_plugin,
            ))
        }

        if let Some(personal_space_admin_plugin) = self.personal_space_admin_plugin {
            attributes.attribute_mut((
                indexer_ids::SPACE_PERSONAL_PLUGIN_ADDRESS,
                personal_space_admin_plugin,
            ))
        }

        Ok(attributes)
    }
}

impl FromAttributes for Space {
    fn from_attributes(mut attributes: Attributes) -> Result<Self, TriplesConversionError> {
        Ok(Self {
            network: attributes.pop(system_ids::NETWORK_ATTRIBUTE)?,
            governance_type: attributes.pop(indexer_ids::SPACE_GOVERNANCE_TYPE)?,
            dao_contract_address: attributes.pop(indexer_ids::SPACE_DAO_ADDRESS)?,
            space_plugin_address: attributes.pop_opt(indexer_ids::SPACE_PLUGIN_ADDRESS)?,
            voting_plugin_address: attributes.pop_opt(indexer_ids::SPACE_VOTING_PLUGIN_ADDRESS)?,
            member_access_plugin: attributes.pop_opt(indexer_ids::SPACE_MEMBER_PLUGIN_ADDRESS)?,
            personal_space_admin_plugin: attributes
                .pop_opt(indexer_ids::SPACE_PERSONAL_PLUGIN_ADDRESS)?,
        })
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub enum SpaceGovernanceType {
    #[default]
    Public,
    Personal,
}

impl From<SpaceGovernanceType> for Value {
    fn from(governance_type: SpaceGovernanceType) -> Self {
        match governance_type {
            SpaceGovernanceType::Public => Value::text("Public".to_string()),
            SpaceGovernanceType::Personal => Value::text("Personal".to_string()),
        }
    }
}

impl TryFrom<Value> for SpaceGovernanceType {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.value.as_str() {
            "Public" => Ok(SpaceGovernanceType::Public),
            "Personal" => Ok(SpaceGovernanceType::Personal),
            _ => Err(format!(
                "Invalid SpaceGovernanceType value: {}",
                value.value
            )),
        }
    }
}

pub struct SpaceBuilder {
    id: String,
    network: String,
    governance_type: SpaceGovernanceType,
    dao_contract_address: String,
    space_plugin_address: Option<String>,
    voting_plugin_address: Option<String>,
    member_access_plugin: Option<String>,
    personal_space_admin_plugin: Option<String>,
}

impl SpaceBuilder {
    pub fn new(id: &str, dao_contract_address: &str) -> Self {
        Self {
            id: id.to_string(),
            network: network_ids::GEO.to_string(),
            governance_type: SpaceGovernanceType::Public,
            dao_contract_address: checksum_address(dao_contract_address),
            space_plugin_address: None,
            voting_plugin_address: None,
            member_access_plugin: None,
            personal_space_admin_plugin: None,
        }
    }

    pub fn network(mut self, network: String) -> Self {
        self.network = network;
        self
    }

    pub fn governance_type(mut self, governance_type: SpaceGovernanceType) -> Self {
        self.governance_type = governance_type;
        self
    }

    pub fn dao_contract_address(mut self, dao_contract_address: &str) -> Self {
        self.dao_contract_address = checksum_address(dao_contract_address);
        self
    }

    pub fn space_plugin_address(mut self, space_plugin_address: &str) -> Self {
        self.space_plugin_address = Some(checksum_address(space_plugin_address));
        self
    }

    pub fn voting_plugin_address(mut self, voting_plugin_address: &str) -> Self {
        self.voting_plugin_address = Some(checksum_address(voting_plugin_address));
        self
    }

    pub fn member_access_plugin(mut self, member_access_plugin: &str) -> Self {
        self.member_access_plugin = Some(checksum_address(member_access_plugin));
        self
    }

    pub fn personal_space_admin_plugin(mut self, personal_space_admin_plugin: &str) -> Self {
        self.personal_space_admin_plugin = Some(checksum_address(personal_space_admin_plugin));
        self
    }

    pub fn build(self) -> Entity<Space> {
        Entity::new(
            &self.id,
            Space {
                network: self.network,
                governance_type: self.governance_type,
                dao_contract_address: self.dao_contract_address,
                space_plugin_address: self.space_plugin_address,
                voting_plugin_address: self.voting_plugin_address,
                member_access_plugin: self.member_access_plugin,
                personal_space_admin_plugin: self.personal_space_admin_plugin,
            },
        )
        .with_type(system_ids::SPACE_TYPE)
    }
}

/// Parent space relation (for subspaces).
/// Space > PARENT_SPACE > Space
#[derive(Clone)]
pub struct ParentSpace;

impl ParentSpace {
    pub fn generate_id(space_id: &str, parent_space_id: &str) -> String {
        ids::create_id_from_unique_string(format!("PARENT_SPACE:{space_id}:{parent_space_id}"))
    }

    pub fn new(space_id: &str, parent_space_id: &str) -> Relation<Self> {
        Relation::new(
            Self::generate_id(space_id, parent_space_id),
            space_id,
            parent_space_id,
            indexer_ids::PARENT_SPACE,
            "0",
            Self,
        )
    }

    /// Delete a relation between a space and its parent space.
    pub async fn remove(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: &str,
        parent_space_id: &str,
    ) -> Result<(), DatabaseError> {
        relation::delete_one(
            neo4j,
            block,
            ParentSpace::generate_id(space_id, parent_space_id),
            indexer_ids::INDEXER_SPACE_ID,
            "0",
        )
        .send()
        .await
    }
}

impl mapping::IntoAttributes for ParentSpace {
    fn into_attributes(self) -> Result<mapping::Attributes, mapping::TriplesConversionError> {
        Ok(mapping::Attributes::default())
    }
}

impl FromAttributes for ParentSpace {
    fn from_attributes(
        _attributes: mapping::Attributes,
    ) -> Result<Self, mapping::TriplesConversionError> {
        Ok(Self {})
    }
}
