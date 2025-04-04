use futures::TryStreamExt;
use grc20_core::{mapping::EntityNode, system_ids};
use grc20_sdk::models::property;
use juniper::{graphql_object, Executor, FieldResult, ScalarValue};

use crate::context::KnowledgeGraph;

use super::{AttributeFilter, Entity, EntityRelationFilter, EntityVersion, Relation, Triple};

#[derive(Debug)]
pub struct Property {
    entity: Entity,
}

impl Property {
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
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl Property {
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

    /// Value type of the property
    async fn value_type<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Option<Entity>> {
        // let value_type = self
        //     .entity
        //     .node
        //     .get_outbound_relations(
        //         &executor.context().0,
        //         self.space_id(),
        //         self.entity.space_version.clone(),
        //     )
        //     .relation_type(prop_filter::value(system_ids::VALUE_TYPE_ATTRIBUTE))
        //     .limit(1)
        //     .send()
        //     .await?;
        tracing::info!("Fetching value type for property {}", self.entity.id());

        let value_type = property::get_outbound_relations(
            &executor.context().0,
            system_ids::VALUE_TYPE_ATTRIBUTE,
            self.entity.id(),
            self.space_id(),
            self.entity.space_version.clone(),
            Some(1),
            None,
            self.entity.strict,
        )
        .await?
        .try_collect::<Vec<_>>()
        .await?;

        if let Some(value_type) = value_type.first() {
            Ok(Entity::load(
                &executor.context().0,
                &value_type.to,
                self.space_id().to_string(),
                self.entity.space_version.clone(),
                self.entity.strict,
            )
            .await?)
        } else {
            Ok(None)
        }
    }

    /// Value type of the property
    async fn relation_value_type<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Option<Entity>> {
        // let rel_value_type = self
        //     .entity
        //     .node
        //     .get_outbound_relations(
        //         &executor.context().0,
        //         self.space_id(),
        //         self.entity.space_version.clone(),
        //     )
        //     .relation_type(prop_filter::value(system_ids::RELATION_VALUE_RELATIONSHIP_TYPE))
        //     .limit(1)
        //     .send()
        //     .await?;
        tracing::info!(
            "Fetching relation value type for property {}",
            self.entity.id()
        );

        let rel_value_type = property::get_outbound_relations(
            &executor.context().0,
            system_ids::RELATION_VALUE_RELATIONSHIP_TYPE,
            self.entity.id(),
            self.space_id(),
            self.entity.space_version.clone(),
            Some(1),
            None,
            self.entity.strict,
        )
        .await?
        .try_collect::<Vec<_>>()
        .await?;

        // pin_mut!(rel_value_type);

        if let Some(value_type) = rel_value_type.first() {
            Ok(Entity::load(
                &executor.context().0,
                &value_type.to,
                self.space_id().to_string(),
                self.entity.space_version.clone(),
                self.entity.strict,
            )
            .await?)
        } else {
            Ok(None)
        }
    }
}
