use juniper::{graphql_object, Executor, ScalarValue};

use sdk::mapping;

use crate::{
    context::KnowledgeGraph,
    schema::{Entity, Relation, RelationFilter},
};

use super::EntityFilter;

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
        r#where: Option<EntityFilter>,
    ) -> Vec<Entity> {
        // let query = QueryMapper::default().select_root_node(&id, &executor.look_ahead()).build();
        // tracing::info!("Query: {}", query);

        match r#where {
            Some(r#where) => mapping::Entity::<mapping::Triples>::find_many(
                &executor.context().0,
                Some(r#where.into()),
            )
            .await
            .expect("Failed to find entities")
            .into_iter()
            .map(Entity::from)
            .collect::<Vec<_>>(),
            _ => mapping::Entity::<mapping::Triples>::find_many(&executor.context().0, None)
                .await
                .expect("Failed to find entities")
                .into_iter()
                .map(Entity::from)
                .collect::<Vec<_>>(),
        }
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
        // version_id: Option<String>,
        filter: Option<RelationFilter>,
    ) -> Vec<Relation> {
        match filter {
            Some(RelationFilter {
                relation_types: Some(types),
            }) if !types.is_empty() => {
                mapping::Relation::<mapping::Triples>::find_by_relation_types(
                    &executor.context().0,
                    &types,
                    &space_id,
                )
                .await
                .expect("Failed to find relations")
                .into_iter()
                .map(|rel| rel.into())
                .collect::<Vec<_>>()
            }
            _ => mapping::Relation::<mapping::Triples>::find_many(
                &executor.context().0,
                Some(mapping::RelationFilter {
                    space_id: Some(space_id),
                    ..Default::default()
                }),
            )
            .await
            .expect("Failed to find relations")
            .into_iter()
            .map(|rel| rel.into())
            .collect::<Vec<_>>(),
        }
    }
}
