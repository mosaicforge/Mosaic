use juniper::{graphql_object, Executor, ScalarValue};

use sdk::mapping::{self, entity_node, relation_node, Query as _};

use crate::{
    context::KnowledgeGraph,
    schema::{Entity, Relation, RelationFilter},
};

use super::{entity_order_by::OrderDirection, EntityFilter};

#[derive(Clone)]
pub struct Query;

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl Query {
    /// Returns a single entity identified by its ID and space ID
    async fn entity<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        id: String,
        space_id: String,
        // version_id: Option<String>,
    ) -> Option<Entity> {
        Entity::load(&executor.context().0, id, space_id, None)
            .await
    }

    /// Returns multiple entities according to the provided space ID and filter
    async fn entities<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        space_id: String,
        order_by: Option<String>,
        order_direction: Option<OrderDirection>,
        r#where: Option<EntityFilter>,
    ) -> Vec<Entity> {
        let mut query = entity_node::find_many(&executor.context().0);

        if let Some(r#where) = r#where {
            query = r#where.apply_filter(query);
        }

        // if let Some(order_by) = order_by {
        //     query = query.order_by(&order_by);
        // }

        // if let Some(order_direction) = order_direction {
        //     query = query.order_direction(order_direction.into());
        // }

        query
            .send()
            .await
            .expect("Failed to find entities")
            .into_iter()
            .map(|entity| Entity::new(entity, space_id.clone(), None))
            .collect::<Vec<_>>()
    }

    /// Returns a single relation identified by its ID and space ID
    async fn relation<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        id: String,
        space_id: String,
        // version_id: Option<String>,
    ) -> Option<Relation> {
        Relation::load(&executor.context().0, id, space_id, None)
            .await
    }

    /// Returns multiple relations according to the provided space ID and filter
    async fn relations<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        space_id: String,
        order_by: Option<String>,
        order_direction: Option<OrderDirection>,
        r#where: Option<RelationFilter>,
    ) -> Vec<Relation> {
        let mut query = relation_node::find_many(&executor.context().0);

        if let Some(r#where) = r#where {
            query = r#where.apply_filter(query);
        }
        
        query
            .send()
            .await
            .expect("Failed to find relations")
            .into_iter()
            .map(|relation| Relation::new(relation, space_id.clone(), None))
            .collect()
    }
}
