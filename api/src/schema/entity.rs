use futures::TryStreamExt;
use grc20_sdk::models::property;
use juniper::{graphql_object, Executor, FieldResult, ScalarValue};

use grc20_core::{
    mapping::{
        entity_node,
        query_utils::{prop_filter, Query, QueryStream},
        triple, EntityNode,
    },
    neo4rs, system_ids,
};

use crate::{
    context::KnowledgeGraph,
    schema::{Relation, Triple},
};

use super::{AttributeFilter, EntityRelationFilter, EntityVersion};

#[derive(Debug)]
pub struct Entity {
    pub node: EntityNode,
    pub space_id: String,
    pub space_version: Option<String>,
    pub strict: bool,
}

impl Entity {
    pub fn new(
        node: EntityNode,
        space_id: String,
        space_version: Option<String>,
        strict: bool,
    ) -> Self {
        Self {
            node,
            space_id,
            space_version,
            strict,
        }
    }

    pub async fn load(
        neo4j: &neo4rs::Graph,
        id: impl Into<String>,
        space_id: impl Into<String>,
        space_version: Option<String>,
        strict: bool,
    ) -> FieldResult<Option<Self>> {
        let id = id.into();
        let space_id = space_id.into();

        Ok(entity_node::find_one(neo4j, id)
            .send()
            .await?
            .map(|node| Entity::new(node, space_id, space_version, strict)))
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
/// Entity object
impl Entity {
    /// Entity ID
    pub fn id(&self) -> &str {
        &self.node.id
    }

    /// The space ID of the entity (note: the same entity can exist in multiple spaces)
    pub fn space_id(&self) -> &str {
        &self.space_id
    }

    pub fn created_at(&self) -> String {
        self.node.system_properties.created_at.to_rfc3339()
    }

    pub fn created_at_block(&self) -> &str {
        &self.node.system_properties.created_at_block
    }

    pub fn updated_at(&self) -> String {
        self.node.system_properties.updated_at.to_rfc3339()
    }

    pub fn updated_at_block(&self) -> &str {
        &self.node.system_properties.updated_at_block
    }

    /// Entity name (if available)
    pub async fn name<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Option<String>> {
        Ok(property::get_triple(
            &executor.context().neo4j,
            system_ids::NAME_ATTRIBUTE,
            &self.node.id,
            &self.space_id,
            self.space_version.clone(),
            self.strict,
        )
        .await?
        .map(|triple| triple.value.value))
    }

    /// Entity description (if available)
    pub async fn description<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Option<String>> {
        Ok(property::get_triple(
            &executor.context().neo4j,
            system_ids::DESCRIPTION_ATTRIBUTE,
            &self.node.id,
            &self.space_id,
            self.space_version.clone(),
            self.strict,
        )
        .await?
        .map(|triple| triple.value.value))
    }

    /// Entity cover (if available)
    pub async fn cover<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Option<String>> {
        Ok(property::get_triple(
            &executor.context().neo4j,
            system_ids::COVER_ATTRIBUTE,
            &self.node.id,
            &self.space_id,
            self.space_version.clone(),
            self.strict,
        )
        .await?
        .map(|triple| triple.value.value))
    }

    /// Entity blocks (if available)
    pub async fn blocks<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Vec<Entity>> {
        let types_rel = self
            .node
            .get_outbound_relations(
                &executor.context().neo4j,
                &self.space_id,
                self.space_version.clone(),
            )
            .relation_type(prop_filter::value(system_ids::BLOCKS))
            .send()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        Ok(entity_node::find_many(&executor.context().neo4j)
            .id(prop_filter::value_in(
                types_rel.into_iter().map(|rel| rel.to).collect(),
            ))
            .send()
            .await?
            .map_ok(|node| {
                Entity::new(
                    node,
                    self.space_id.clone(),
                    self.space_version.clone(),
                    self.strict,
                )
            })
            .try_collect::<Vec<_>>()
            .await?)
    }

    /// Types of the entity (which are entities themselves)
    pub async fn types<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Vec<Entity>> {
        let types_rel = self
            .node
            .get_outbound_relations(
                &executor.context().neo4j,
                &self.space_id,
                self.space_version.clone(),
            )
            .relation_type(prop_filter::value(system_ids::TYPES_ATTRIBUTE))
            .send()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        Ok(entity_node::find_many(&executor.context().neo4j)
            .id(prop_filter::value_in(
                types_rel.into_iter().map(|rel| rel.to).collect(),
            ))
            .send()
            .await?
            .map_ok(|node| {
                Entity::new(
                    node,
                    self.space_id.clone(),
                    self.space_version.clone(),
                    self.strict,
                )
            })
            .try_collect::<Vec<_>>()
            .await?)
    }

    // TODO: Add entity attributes filtering
    /// Attributes of the entity
    pub async fn attributes<S: ScalarValue>(
        &self,
        executor: &'_ Executor<'_, '_, KnowledgeGraph, S>,
        _filter: Option<AttributeFilter>,
    ) -> FieldResult<Vec<Triple>> {
        let mut query = triple::find_many(&executor.context().neo4j)
            .entity_id(prop_filter::value(&self.node.id))
            .space_id(prop_filter::value(&self.space_id));

        if let Some(version) = &self.space_version {
            query = query.space_version(version);
        }

        Ok(query
            .send()
            .await?
            .map_ok(|triple| Triple::new(triple, self.space_id.clone(), self.space_version.clone()))
            .try_collect::<Vec<_>>()
            .await?)
    }

    /// Relations outgoing from the entity
    pub async fn relations<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
        r#where: Option<EntityRelationFilter>,
    ) -> FieldResult<Vec<Relation>> {
        let mut base_query = self.node.get_outbound_relations(
            &executor.context().neo4j,
            &self.space_id,
            self.space_version.clone(),
        );

        if let Some(filter) = r#where {
            base_query = filter.apply_filter(base_query);
        }

        Ok(base_query
            .send()
            .await?
            .map_ok(|relation| {
                Relation::new(
                    relation,
                    self.space_id.clone(),
                    self.space_version.clone(),
                    self.strict,
                )
            })
            .try_collect::<Vec<_>>()
            .await?)
    }

    // TODO: Add version filtering (e.g.: time range, edit author)
    /// Versions of the entity, ordered chronologically
    pub async fn versions<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Vec<EntityVersion>> {
        Ok(self
            .node
            .versions(&executor.context().neo4j)
            .space_id(prop_filter::value(&self.space_id))
            .send()
            .await?
            .into_iter()
            .map(|version| {
                EntityVersion::new(
                    version.id,
                    version.entity_id,
                    version.index,
                    self.space_id.clone(),
                )
            })
            .collect())
    }
}
