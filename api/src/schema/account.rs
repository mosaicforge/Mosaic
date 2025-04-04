use juniper::{graphql_object, FieldResult, ScalarValue};

use grc20_core::{
    indexer_ids,
    mapping::{query_utils::Query, Entity},
    neo4rs,
};
use grc20_sdk::models::{account, Account as SdkAccount};

use crate::context::KnowledgeGraph;

pub struct Account {
    entity: Entity<SdkAccount>,
}

impl Account {
    pub fn new(entity: Entity<SdkAccount>) -> Self {
        Self { entity }
    }

    pub async fn load(neo4j: &neo4rs::Graph, id: impl Into<String>) -> FieldResult<Option<Self>> {
        let id = id.into();

        Ok(account::find_one(neo4j, &id, indexer_ids::INDEXER_SPACE_ID)
            .send()
            .await?
            .map(Account::new))
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl Account {
    /// Account ID
    fn id(&self) -> &str {
        self.entity.id()
    }

    /// Ethereum address of the account
    fn address(&self) -> &str {
        &self.entity.attributes.address
    }
}
