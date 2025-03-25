use juniper::{graphql_object, FieldResult, ScalarValue};

use sdk::{
    mapping::{query_utils::Query, Entity},
    models::Account as SdkAccount,
    neo4rs,
};

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

        Ok(SdkAccount::find_one(neo4j, &id)
            .send()
            .await?
            .map(Account::new))
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl Account {
    fn id(&self) -> &str {
        &self.entity.id
    }

    fn address(&self) -> &str {
        &self.entity.attributes.address
    }
}
