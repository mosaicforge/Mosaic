use futures::{StreamExt, TryStreamExt};
use juniper::{graphql_object, Executor, FieldResult, ScalarValue};

use sdk::{
    mapping::{query_utils::{Query, QueryStream}, Entity},
    models::Space as SdkSpace,
    neo4rs,
};

use crate::context::KnowledgeGraph;

pub struct Space {
    entity: Entity<SdkSpace>,
}

impl Space {
    pub fn new(entity: Entity<SdkSpace>) -> Self {
        Self { entity }
    }

    pub async fn load(
        neo4j: &neo4rs::Graph,
        id: impl Into<String>,
    ) -> FieldResult<Option<Self>> {
        let id = id.into();

        Ok(SdkSpace::find_one(neo4j, &id)
            .send()
            .await?
            .map(Space::new))
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl Space {
    fn id(&self) -> &str {
        &self.entity.id
    }

    fn network(&self) -> &str {
        &self.entity.attributes.network
    }

    fn governance_type(&self) -> &str {
        match self.entity.attributes.governance_type {
            sdk::models::space::SpaceGovernanceType::Public => "Public",
            sdk::models::space::SpaceGovernanceType::Personal => "Personal",
        }
    }

    fn dao_contract_address(&self) -> &str {
        &self.entity.attributes.dao_contract_address
    }

    fn space_plugin_address(&self) -> Option<&str> {
        self.entity.attributes.space_plugin_address.as_deref()
    }

    fn voting_plugin_address(&self) -> Option<&str> {
        self.entity.attributes.voting_plugin_address.as_deref()
    }

    fn member_access_plugin(&self) -> Option<&str> {
        self.entity.attributes.member_access_plugin.as_deref()
    }

    fn personal_space_admin_plugin(&self) -> Option<&str> {
        self.entity.attributes.personal_space_admin_plugin.as_deref()
    }

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
            .map_ok(|entity| super::Account::new(entity))
            .try_collect::<Vec<_>>()
            .await?)
    }

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
            .map_ok(|entity| super::Account::new(entity))
            .try_collect::<Vec<_>>()
            .await?)
    }

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

        Ok(query
            .send()
            .await?
            .map_ok(|entity| Space::new(entity))
            .try_collect::<Vec<_>>()
            .await?)
    }

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

        Ok(query
            .send()
            .await?
            .map_ok(|entity| Space::new(entity))
            .try_collect::<Vec<_>>()
            .await?)
    }
}
