use juniper::{graphql_object, Executor, ScalarValue};

use sdk::mapping;

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
        // let query = QueryMapper::default().select_root_node(&id, &executor.look_ahead()).build();
        // tracing::info!("Query: {}", query);

        mapping::Entity::<mapping::Triples>::find_by_id(&executor.context().0, &id, &space_id)
            .await
            .expect("Failed to find entity")
            .map(Entity::from)
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
        let mut base_query = mapping::entity_queries::FindMany::new("n").space_id(&space_id);

        if let Some(r#where) = r#where {
            base_query = r#where.add_to_entity_query(base_query);

            if let Some(order_by) = order_by {
                base_query = base_query.order_by(&order_by);
            }

            if let Some(order_direction) = order_direction {
                base_query = base_query.order_direction(order_direction.into());
            }
        }

        mapping::Entity::<mapping::Triples>::find_many(&executor.context().0, Some(base_query))
            .await
            .expect("Failed to find entities")
            .into_iter()
            .map(Entity::from)
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
        mapping::Relation::<mapping::Triples>::find_by_id(&executor.context().0, &id, &space_id)
            .await
            .expect("Failed to find relation")
            .map(|rel| rel.into())
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
        let mut base_query = mapping::relation_queries::FindMany::new("r").space_id(&space_id);

        if let Some(r#where) = r#where {
            base_query = r#where.add_to_relation_query(base_query);

            if let Some(order_by) = order_by {
                base_query = base_query.order_by(&order_by);
            }

            if let Some(order_direction) = order_direction {
                base_query = base_query.order_direction(order_direction.into());
            }
        }

        mapping::Relation::<mapping::Triples>::find_many(&executor.context().0, Some(base_query))
            .await
            .expect("Failed to find relations")
            .into_iter()
            .map(|rel| rel.into())
            .collect::<Vec<_>>()
    }
}
