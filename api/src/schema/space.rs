use futures::TryStreamExt;
use juniper::{graphql_object, Executor, FieldResult, GraphQLEnum, ScalarValue};


use grc20_core::{
    indexer_ids,
    mapping::{self, query_utils::{Query, QueryStream}},
    neo4rs,
};
use grc20_sdk::models::{space::{self, ParentSpacesQuery, SubspacesQuery}, Space as SdkSpace};

use crate::context::KnowledgeGraph;

use super::{Account, Entity};

pub struct Space {
    entity: mapping::Entity<SdkSpace>,
}

impl Space {
    pub fn new(entity: mapping::Entity<SdkSpace>) -> Self {
        Self { entity }
    }

    pub async fn load(neo4j: &neo4rs::Graph, id: impl Into<String>) -> FieldResult<Option<Self>> {
        let id = id.into();

        Ok(space::find_one(neo4j, &id, indexer_ids::INDEXER_SPACE_ID)
            .send()
            .await?
            .map(Space::new))
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
        &self.entity.id
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
        first: Option<i32>,
        skip: Option<i32>,
    ) -> FieldResult<Vec<super::Account>> {
        let mut query = SdkSpace::members(&executor.context().0, &self.entity.id);

        if let Some(first) = first {
            if first > 1000 {
                return Err("Cannot query more than 1000 members at once".into());
            }
            query = query.limit(first as usize);
        }

        if let Some(skip) = skip {
            query = query.skip(skip as usize);
        }

        Ok(query
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
        first: Option<i32>,
        skip: Option<i32>,
    ) -> FieldResult<Vec<super::Account>> {
        let mut query = SdkSpace::editors(&executor.context().0, &self.entity.id);

        if let Some(first) = first {
            if first > 1000 {
                return Err("Cannot query more than 1000 editors at once".into());
            }
            query = query.limit(first as usize);
        }

        if let Some(skip) = skip {
            query = query.skip(skip as usize);
        }

        Ok(query
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
        first: Option<i32>,
        skip: Option<i32>,
    ) -> FieldResult<Vec<Space>> {
        let mut query = SdkSpace::parent_spaces(&executor.context().0, &self.entity.id);

        if let Some(first) = first {
            if first > 1000 {
                return Err("Cannot query more than 1000 parent spaces at once".into());
            }
            query = query.limit(first as usize);
        }

        if let Some(skip) = skip {
            query = query.skip(skip as usize);
        }

        Ok(<ParentSpacesQuery as QueryStream<mapping::Entity<grc20_sdk::models::Space>>>::send(query)
            .await?
            .map_ok(Space::new)
            .try_collect::<Vec<_>>()
            .await?)
    }

    /// Subspaces of this space
    async fn subspaces<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        first: Option<i32>,
        skip: Option<i32>,
    ) -> FieldResult<Vec<Space>> {
        let mut query = SdkSpace::subspaces(&executor.context().0, &self.entity.id);

        if let Some(first) = first {
            if first > 1000 {
                return Err("Cannot query more than 1000 subspaces at once".into());
            }
            query = query.limit(first as usize);
        }

        if let Some(skip) = skip {
            query = query.skip(skip as usize);
        }

        Ok(<SubspacesQuery as QueryStream<mapping::Entity<grc20_sdk::models::Space>>>::send(query)
            .await?
            .map_ok(Space::new)
            .try_collect::<Vec<_>>()
            .await?)
    }

    async fn types<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        strict: Option<bool>,
        first: Option<i32>,
        skip: Option<i32>,
    ) -> FieldResult<Vec<Entity>> {
        let types = grc20_sdk::models::space::schema_types::space_schema_types(
            &executor.context().0,
            &self.entity.id,
            strict.unwrap_or(true),
        )
        .await?;

        Ok(types
            .into_iter()
            .skip(skip.unwrap_or(0) as usize)
            .take(first.unwrap_or(1000) as usize)
            .map(|node| Entity::new(node, self.entity.id.clone(), None))
            .collect())
    }
}
