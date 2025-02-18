use juniper::{graphql_object, Executor, FieldResult, ScalarValue};

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
        version_id: Option<String>,
    ) -> FieldResult<Option<Entity>> {
        tracing::info!("version_id: {:?}", version_id);
        let version_index = if let Some(version_id) = version_id {
            mapping::get_version_index(&executor.context().0, version_id).await?
        } else {
            None
        };
        tracing::info!("version_index: {:?}", version_index);

        Entity::load(&executor.context().0, id, space_id, version_index).await
    }

    // TODO: Add order_by and order_direction
    /// Returns multiple entities according to the provided space ID and filter
    async fn entities<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        space_id: String,
        _order_by: Option<String>,
        _order_direction: Option<OrderDirection>,
        r#where: Option<EntityFilter>,
    ) -> Vec<Entity> {
        let mut query = entity_node::find_many(&executor.context().0);

        if let Some(r#where) = r#where {
            query = query.with_filter(r#where.into());
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
        version_id: Option<String>,
    ) -> FieldResult<Option<Relation>> {
        let version_index = if let Some(version_id) = version_id {
            mapping::get_version_index(&executor.context().0, version_id).await?
        } else {
            None
        };

        Relation::load(&executor.context().0, id, space_id, version_index).await
    }

    // TODO: Add order_by and order_direction
    /// Returns multiple relations according to the provided space ID and filter
    async fn relations<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        space_id: String,
        _order_by: Option<String>,
        _order_direction: Option<OrderDirection>,
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
