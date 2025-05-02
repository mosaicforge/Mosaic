use futures::{StreamExt, TryStreamExt};
use juniper::{graphql_object, Executor, FieldResult, GraphQLEnum, ScalarValue};

use grc20_core::{
    entity::EntityNode,
    error::DatabaseError,
    indexer_ids,
    mapping::{
        self,
        aggregation::SpaceRanking,
        entity, prop_filter,
        query_utils::{Query, QueryStream},
    },
    neo4rs,
};
use grc20_sdk::models::{self, space, Space as SdkSpace};

use crate::context::KnowledgeGraph;

use super::{entity_order_by::OrderDirection, Account, Entity, EntityFilter, SchemaType};

pub struct Space {
    entity: mapping::Entity<SdkSpace>,
    version: Option<String>,
    parent_spaces: Vec<SpaceRanking>,
    subspaces: Vec<SpaceRanking>,
}

impl Space {
    pub fn new(
        entity: mapping::Entity<SdkSpace>,
        version: Option<String>,
        parent_spaces: Vec<SpaceRanking>,
        subspaces: Vec<SpaceRanking>,
    ) -> Self {
        Self {
            entity,
            version,
            parent_spaces,
            subspaces,
        }
    }

    pub async fn from_entity(
        neo4j: &neo4rs::Graph,
        entity: mapping::Entity<SdkSpace>,
        version: Option<String>,
    ) -> Result<Self, DatabaseError> {
        let parent_spaces = models::space::parent_spaces(neo4j, entity.id())
            .max_depth(None)
            .send()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        let subspaces = models::space::subspaces(neo4j, entity.id())
            .max_depth(None)
            .send()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        Ok(Self::new(entity, version, parent_spaces, subspaces))
    }

    pub async fn load(
        neo4j: &neo4rs::Graph,
        id: impl Into<String>,
        version: Option<String>,
    ) -> Result<Option<Self>, DatabaseError> {
        let id = id.into();

        if let Some(space) = space::find_one(neo4j, &id, indexer_ids::INDEXER_SPACE_ID)
            .send()
            .await?
        {
            let parent_spaces = models::space::parent_spaces(neo4j, &id)
                .max_depth(None)
                .send()
                .await?
                .skip(1) // The returned spaces contain the current space
                .try_collect::<Vec<_>>()
                .await?;

            let subspaces = models::space::subspaces(neo4j, &id)
                .max_depth(None)
                .send()
                .await?
                .skip(1) // The returned spaces contain the current space
                .try_collect::<Vec<_>>()
                .await?;

            Ok(Some(Self::new(
                space,
                version.clone(),
                parent_spaces,
                subspaces,
            )))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone, Debug, GraphQLEnum)]
pub enum SpaceGovernanceType {
    Public,
    Personal,
}

impl From<grc20_sdk::models::space::SpaceGovernanceType> for SpaceGovernanceType {
    fn from(governance_type: grc20_sdk::models::space::SpaceGovernanceType) -> Self {
        match governance_type {
            grc20_sdk::models::space::SpaceGovernanceType::Public => SpaceGovernanceType::Public,
            grc20_sdk::models::space::SpaceGovernanceType::Personal => {
                SpaceGovernanceType::Personal
            }
        }
    }
}

impl From<SpaceGovernanceType> for grc20_sdk::models::space::SpaceGovernanceType {
    fn from(governance_type: SpaceGovernanceType) -> Self {
        match governance_type {
            SpaceGovernanceType::Public => grc20_sdk::models::space::SpaceGovernanceType::Public,
            SpaceGovernanceType::Personal => {
                grc20_sdk::models::space::SpaceGovernanceType::Personal
            }
        }
    }
}

impl From<&SpaceGovernanceType> for grc20_sdk::models::space::SpaceGovernanceType {
    fn from(governance_type: &SpaceGovernanceType) -> Self {
        match governance_type {
            SpaceGovernanceType::Public => grc20_sdk::models::space::SpaceGovernanceType::Public,
            SpaceGovernanceType::Personal => {
                grc20_sdk::models::space::SpaceGovernanceType::Personal
            }
        }
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl Space {
    /// Space ID
    fn id(&self) -> &str {
        self.entity.id()
    }

    /// Network of the space
    fn network(&self) -> &str {
        &self.entity.attributes.network
    }

    /// Governance type of the space (Public or Personal)
    fn governance_type(&self) -> SpaceGovernanceType {
        self.entity.attributes.governance_type.clone().into()
    }

    /// DAO contract address of the space
    fn dao_contract_address(&self) -> &str {
        &self.entity.attributes.dao_contract_address
    }

    /// Space plugin address (if available)
    fn space_plugin_address(&self) -> Option<&str> {
        self.entity.attributes.space_plugin_address.as_deref()
    }

    /// Voting plugin address (if available)
    fn voting_plugin_address(&self) -> Option<&str> {
        self.entity.attributes.voting_plugin_address.as_deref()
    }

    /// Member access plugin address (if available)
    fn member_access_plugin(&self) -> Option<&str> {
        self.entity.attributes.member_access_plugin.as_deref()
    }

    /// Personal space admin plugin address (if available)
    fn personal_space_admin_plugin(&self) -> Option<&str> {
        self.entity
            .attributes
            .personal_space_admin_plugin
            .as_deref()
    }

    // fn updated_at(&self) -> &str {
    //     &self.entity.updated_at
    // }

    // fn created_at(&self) -> &str {
    //     &self.entity.created_at
    // }

    // fn updated_at_block(&self) -> i32 {
    //     self.entity.updated_at_block
    // }

    // fn created_at_block(&self) -> i32 {
    //     self.entity.created_at_block
    // }

    /// Members of the space
    async fn members<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        #[graphql(default = 100)] first: i32,
        #[graphql(default = 0)] skip: i32,
    ) -> FieldResult<Vec<super::Account>> {
        let query = models::space::members(&executor.context().neo4j, self.entity.id());

        if first > 1000 {
            return Err("Cannot query more than 1000 relations at once".into());
        }

        Ok(query
            .limit(first as usize)
            .skip(skip as usize)
            .send()
            .await?
            .map_ok(Account::new)
            .try_collect::<Vec<_>>()
            .await?)
    }

    /// Editors of the space
    async fn editors<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        #[graphql(default = 100)] first: i32,
        #[graphql(default = 0)] skip: i32,
    ) -> FieldResult<Vec<super::Account>> {
        let query = models::space::editors(&executor.context().neo4j, self.entity.id());

        if first > 1000 {
            return Err("Cannot query more than 1000 relations at once".into());
        }

        Ok(query
            .limit(first as usize)
            .skip(skip as usize)
            .send()
            .await?
            .map_ok(Account::new)
            .try_collect::<Vec<_>>()
            .await?)
    }

    /// Parent spaces of this space
    async fn parent_spaces<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        #[graphql(default = 100)] first: i32,
        #[graphql(default = 0)] skip: i32,
    ) -> FieldResult<Vec<Space>> {
        let query = models::space::parent_spaces(&executor.context().neo4j, self.entity.id());

        if first > 1000 {
            return Err("Cannot query more than 1000 relations at once".into());
        }

        Ok(query
            .limit(first as usize)
            .skip(skip as usize)
            .send()
            .await?
            .skip(1) // The returned spaces contain the current space
            .and_then(|ranking| Space::load(&executor.context().neo4j, ranking.space_id, None))
            .filter_map(|space| async move { space.transpose() })
            .try_collect::<Vec<_>>()
            .await?)
    }

    /// Subspaces of this space
    async fn subspaces<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        #[graphql(default = 100)] first: i32,
        #[graphql(default = 0)] skip: i32,
    ) -> FieldResult<Vec<Space>> {
        let query = models::space::subspaces(&executor.context().neo4j, self.entity.id());

        if first > 1000 {
            return Err("Cannot query more than 1000 relations at once".into());
        }

        Ok(query
            .limit(first as usize)
            .skip(skip as usize)
            .send()
            .await?
            .skip(1) // The returned spaces contain the current space
            .and_then(|ranking| Space::load(&executor.context().neo4j, ranking.space_id, None))
            .filter_map(|space| async move { space.transpose() })
            .try_collect::<Vec<_>>()
            .await?)
    }

    async fn types<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        #[graphql(default = 100)] first: i32,
        #[graphql(default = 0)] skip: i32,
        #[graphql(default = true)] strict: bool,
    ) -> FieldResult<Vec<SchemaType>> {
        let types = models::space::types(&executor.context().neo4j, self.entity.id())
            .strict(strict)
            .limit(first as usize)
            .skip(skip as usize)
            .send()
            .await?;

        Ok(types
            .map_ok(|node| {
                SchemaType::with_hierarchy(
                    node,
                    self.entity.id().to_string(),
                    self.parent_spaces.clone(),
                    self.subspaces.clone(),
                    None,
                    strict,
                )
            })
            .try_collect()
            .await?)
    }

    async fn r#type<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        id: String,
        #[graphql(default = true)] strict: bool,
    ) -> FieldResult<Option<SchemaType>> {
        let type_ = models::space::r#type(&executor.context().neo4j, self.entity.id(), &id)
            .strict(strict)
            .send()
            .await?;

        if let Some(type_) = type_ {
            Ok(Some(SchemaType::with_hierarchy(
                type_,
                self.entity.id().to_string(),
                self.parent_spaces.clone(),
                self.subspaces.clone(),
                None,
                strict,
            )))
        } else {
            Ok(None)
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn entities<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        order_by: Option<String>,
        order_direction: Option<OrderDirection>,
        r#where: Option<EntityFilter>,
        #[graphql(default = 100)] first: i32,
        #[graphql(default = 0)] skip: i32,
        #[graphql(default = true)] strict: bool,
    ) -> FieldResult<Vec<Entity>> {
        let mut query = entity::find_many::<EntityNode>(&executor.context().neo4j);

        let entity_filter = if let Some(r#where) = r#where {
            mapping::EntityFilter::from(r#where).space_id(prop_filter::value(self.id()))
        } else {
            mapping::EntityFilter::default().space_id(prop_filter::value(self.id()))
        };
        query = query.with_filter(entity_filter);

        match (order_by, order_direction) {
            (Some(order_by), Some(OrderDirection::Asc) | None) => {
                query.order_by_mut(mapping::order_by::asc(order_by));
            }
            (Some(order_by), Some(OrderDirection::Desc)) => {
                query.order_by_mut(mapping::order_by::desc(order_by));
            }
            _ => {}
        }

        if first > 1000 {
            return Err("Cannot query more than 1000 relations at once".into());
        }

        Ok(query
            .limit(first as usize)
            .skip(skip as usize)
            .send()
            .await?
            .map_ok(|entity| {
                Entity::new(entity, self.id().to_owned(), self.version.clone(), strict)
            })
            .try_collect::<Vec<_>>()
            .await?)
    }
}
