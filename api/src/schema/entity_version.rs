use juniper::{graphql_object, Executor, ScalarValue};
use sdk::mapping::{query_utils::prop_filter, triple, EntityNode, Query};

use crate::context::KnowledgeGraph;

use super::{AttributeFilter, Triple};

pub struct EntityVersion {
    pub id: String,
    entity_id: String,
    index: String,
    space_id: String,
}

impl EntityVersion {
    pub fn new(
        id: String,
        entity_id: String,
        index: String,
        space_id: String,
    ) -> Self {
        Self {
            entity_id,
            id,
            index,
            space_id,
        }
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl EntityVersion {
    fn id(&self) -> &str {
        &self.id
    }

    // fn index(&self) -> &str {
    //     &self.index
    // }

    // TODO: Add entity attributes filtering
    /// Attributes of the entity
    async fn attributes<'a, S: ScalarValue>(
        &self,
        _filter: Option<AttributeFilter>,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Triple> {
        let query = triple::find_many(&executor.context().0)
            .entity_id(prop_filter::value(&self.entity_id))
            .space_version(&self.index);

        query
            .send()
            .await
            .expect("Failed to get attributes")
            .into_iter()
            .map(|triple| Triple::new(triple, self.space_id.clone(), Some(self.index.clone())))
            .collect()
    }
}