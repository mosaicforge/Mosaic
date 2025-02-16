use juniper::{graphql_object, Executor, ScalarValue};

use sdk::{
    mapping::{entity_node, query_utils::{prop_filter, Query}, triple, EntityNode},
    neo4rs, system_ids,
};

use crate::{
    context::KnowledgeGraph,
    schema::{Relation, Triple},
};

use super::{AttributeFilter, EntityRelationFilter};

#[derive(Debug)]
pub struct Entity {
    node: EntityNode,
    space_id: String,
    space_version: Option<i64>,
}

impl Entity {
    pub fn new(node: EntityNode, space_id: String, space_version: Option<i64>) -> Self {
        Self {
            node,
            space_id,
            space_version,
        }
    }

    pub async fn load(
        neo4j: &neo4rs::Graph,
        id: impl Into<String>,
        space_id: impl Into<String>,
        space_version: Option<i64>,
    ) -> Option<Self> {
        let id = id.into();
        let space_id = space_id.into();

        entity_node::find_one(neo4j, id)
            .send()
            .await
            .expect("Failed to find entity")
            .map(|node| Entity::new(node, space_id, space_version))
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
/// Entity object
impl Entity {
    /// Entity ID
    fn id(&self) -> &str {
        &self.node.id
    }

    /// Entity name (if available)
    async fn name<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Option<String> {
        triple::find_one(
            &executor.context().0,
            system_ids::NAME_ATTRIBUTE,
            &self.node.id,
            &self.space_id,
            self.space_version,
        )
        .send()
        .await
        .expect("Failed to find name")
        .map(|triple| triple.value.value)
    }

    /// Entity description (if available)
    async fn description<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Option<String> {
        triple::find_one(
            &executor.context().0,
            system_ids::DESCRIPTION_ATTRIBUTE,
            &self.node.id,
            &self.space_id,
            self.space_version,
        )
        .send()
        .await
        .expect("Failed to find name")
        .map(|triple| triple.value.value)
    }

    /// Entity cover (if available)
    async fn cover<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Option<String> {
        triple::find_one(
            &executor.context().0,
            system_ids::COVER_ATTRIBUTE,
            &self.node.id,
            &self.space_id,
            self.space_version,
        )
        .send()
        .await
        .expect("Failed to find name")
        .map(|triple| triple.value.value)
    }

    /// Entity blocks (if available)
    async fn blocks<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Entity> {
        todo!()
    }

    /// Types of the entity (which are entities themselves)
    async fn types<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Entity> {
        todo!()
    }

    /// The space ID of the entity (note: the same entity can exist in multiple spaces)
    fn space_id(&self) -> &str {
        &self.space_id
    }

    fn created_at(&self) -> String {
        self.node.system_properties.created_at.to_rfc3339()
    }

    fn created_at_block(&self) -> &str {
        &self.node.system_properties.created_at_block
    }

    fn updated_at(&self) -> String {
        self.node.system_properties.updated_at.to_rfc3339()
    }

    fn updated_at_block(&self) -> &str {
        &self.node.system_properties.updated_at_block
    }

    /// Attributes of the entity
    async fn attributes<'a, S: ScalarValue>(
        &self,
        filter: Option<AttributeFilter>,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Triple> {
        let mut query = triple::find_many(&executor.context().0)
            .entity_id(prop_filter::value(&self.node.id))
            .space_id(prop_filter::value(&self.space_id));

        if let Some(version) = self.space_version {
            query = query.space_version(version);
        }

        query
            .send()
            .await
            .expect("Failed to get attributes")
            .into_iter()
            .map(|triple| Triple::new(triple, self.space_id.clone(), self.space_version))
            .collect()
    }

    /// Relations outgoing from the entity
    async fn relations<'a, S: ScalarValue>(
        &'a self,
        r#where: Option<EntityRelationFilter>,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Relation> {
        let mut base_query = self.node.get_outbound_relations(
            &executor.context().0,
            &self.space_id,
            self.space_version,
        );

        if let Some(filter) = r#where {
            base_query = filter.apply_filter(base_query);
        }

        base_query
            .send()
            .await
            .expect("Failed to get relations")
            .into_iter()
            .map(|relation| Relation::new(relation, self.space_id.clone(), self.space_version))
            .collect()
    }
}
