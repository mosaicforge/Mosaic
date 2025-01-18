use juniper::{graphql_object, Executor, ScalarValue};

use sdk::mapping;

use crate::{
    context::KnowledgeGraph,
    schema::{Entity, Relation, RelationFilter},
};

use super::{
    entity_order_by::OrderDirection,
    EntityFilter,
};

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
        let mut base_query = mapping::entity_queries::FindMany::new("n")
            .space_id(&space_id);

        if let Some(r#where) = r#where {
            if let Some(id) = r#where.id {
                base_query = base_query.id(&id);
            }

            if let Some(types_contain) = r#where.types_contain {
                base_query = base_query.types_contains(types_contain);
            }

            if let Some(attributes) = r#where.attributes {
                for attr in attributes {
                    if let Some(value) = attr.value {
                        base_query = base_query.attribute(&attr.attribute, &value);
                    }

                    if let Some(value_type) = attr.value_type {
                        base_query = base_query.attribute_value_type(&attr.attribute, &value_type.to_string());
                    }
                }
            }

            if let Some(order_by) = order_by {
                base_query = base_query.order_by(&order_by);
            }

            if let Some(order_direction) = order_direction {
                base_query = base_query.order_direction(order_direction.into());
            }
        }

        mapping::Entity::<mapping::Triples>::find_many(
            &executor.context().0, 
            Some(base_query),
        )
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
        filter: Option<RelationFilter>,
    ) -> Vec<Relation> {
        let mut base_query = mapping::relation_queries::FindMany::new("r")
            .space_id(&space_id);

        if let Some(filter) = filter {
            // if let Some(id) = filter.id {
            //     base_query.id(&id);
            // }

            // if let Some(from_id) = filter.from_id {
            //     base_query.from_id(&from_id);
            // }

            // if let Some(to_id) = filter.to_id {
            //     base_query.to_id(&to_id);
            // }

            if let Some(relation_type) = filter.relation_type {
                base_query = base_query.relation_type(&relation_type);
            }

            if let Some(order_by) = order_by {
                base_query = base_query.order_by(&order_by);
            }

            if let Some(order_direction) = order_direction {
                base_query = base_query.order_direction(order_direction.into());
            }
        }

        mapping::Relation::<mapping::Triples>::find_many(
            &executor.context().0, 
            Some(base_query),
        )
            .await
            .expect("Failed to find relations")
            .into_iter()
            .map(|rel| rel.into())
            .collect::<Vec<_>>()
    }
}
