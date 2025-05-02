use futures::TryStreamExt;
use grc20_core::{
    mapping::{aggregation::SpaceRanking, query_utils::QueryStream, EntityNode, RelationEdge},
    system_ids,
};
use grc20_sdk::models::property;
use juniper::{graphql_object, Executor, FieldResult, ScalarValue};

use crate::context::KnowledgeGraph;

use super::{
    AttributeFilter, Entity, EntityRelationFilter, EntityVersion, Property, Relation, Triple,
};

#[derive(Debug)]
pub struct SchemaType {
    entity: Entity,
}

impl SchemaType {
    pub fn new(
        node: EntityNode,
        space_id: String,
        space_version: Option<String>,
        strict: bool,
    ) -> Self {
        Self {
            entity: Entity::new(node, space_id, space_version, strict),
        }
    }

    pub fn with_hierarchy(
        node: EntityNode,
        space_id: String,
        parent_spaces: Vec<SpaceRanking>,
        subspaces: Vec<SpaceRanking>,
        space_version: Option<String>,
        strict: bool,
    ) -> Self {
        Self {
            entity: Entity::with_hierarchy(node, space_id, parent_spaces, subspaces, space_version, strict),
        }
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
/// SchemaType object
impl SchemaType {
    /// Entity ID
    fn id(&self) -> &str {
        self.entity.id()
    }

    /// The space ID of the entity (note: the same entity can exist in multiple spaces)
    fn space_id(&self) -> &str {
        self.entity.space_id()
    }

    fn created_at(&self) -> String {
        self.entity.created_at()
    }

    fn created_at_block(&self) -> &str {
        self.entity.created_at_block()
    }

    fn updated_at(&self) -> String {
        self.entity.updated_at()
    }

    fn updated_at_block(&self) -> &str {
        self.entity.updated_at_block()
    }

    /// Entity name (if available)
    async fn name<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        #[graphql(default = true)] _strict: bool,
    ) -> FieldResult<Option<String>> {
        self.entity.name(executor).await
    }

    /// Entity description (if available)
    async fn description<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Option<String>> {
        self.entity.description(executor).await
    }

    /// Entity cover (if available)
    async fn cover<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Option<String>> {
        self.entity.cover(executor).await
    }

    /// Entity blocks (if available)
    async fn blocks<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Vec<Entity>> {
        self.entity.blocks(executor).await
    }

    /// Types of the entity (which are entities themselves)
    async fn types<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Vec<Entity>> {
        self.entity.types(executor).await
    }

    // TODO: Add entity attributes filtering
    /// Attributes of the entity
    async fn attributes<S: ScalarValue>(
        &self,
        executor: &'_ Executor<'_, '_, KnowledgeGraph, S>,
        filter: Option<AttributeFilter>,
    ) -> FieldResult<Vec<Triple>> {
        self.entity.attributes(executor, filter).await
    }

    /// Relations outgoing from the entity
    async fn relations<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        r#where: Option<EntityRelationFilter>,
    ) -> FieldResult<Vec<Relation>> {
        self.entity.relations(executor, r#where).await
    }

    // TODO: Add version filtering (e.g.: time range, edit author)
    /// Versions of the entity, ordered chronologically
    async fn versions<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Vec<EntityVersion>> {
        self.entity.versions(executor).await
    }

    /// Properties of the Type
    async fn properties<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        #[graphql(default = 100)] first: i32,
        #[graphql(default = 0)] skip: i32,
    ) -> FieldResult<Vec<Property>> {
        tracing::info!("Fetching properties for type {}", self.entity.id());

        let properties = property::get_outbound_relations::<RelationEdge<EntityNode>>(
            &executor.context().neo4j,
            system_ids::PROPERTIES,
            self.entity.id(),
            self.space_id(),
            self.entity.space_version.clone(),
            Some(first as usize),
            Some(skip as usize),
            self.entity.strict,
        )
        .await?
        .send()
        .await?
        .try_collect::<Vec<_>>()
        .await?;

        Ok(properties
            .into_iter()
            .map(|prop_rel| {
                Property::with_hierarchy(
                    prop_rel.to,
                    self.space_id().to_string(),
                    self.entity.parent_spaces.clone(),
                    self.entity.subspaces.clone(),    
                    self.entity.space_version.clone(),
                    self.entity.strict,
                )
            })
            .collect())
    }
}
