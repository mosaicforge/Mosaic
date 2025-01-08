use chrono::{DateTime, Utc};
use juniper::{graphql_object, Executor, ScalarValue};

use sdk::{mapping, system_ids};

use crate::{
    context::KnowledgeGraph,
    schema::{Relation, Triple},
};

use super::{AttributeFilter, EntityRelationFilter, Options};

#[derive(Debug)]
pub struct Entity {
    pub(crate) id: String,
    pub(crate) types: Vec<String>,
    pub(crate) space_id: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) created_at_block: String,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) updated_at_block: String,
    pub(crate) attributes: Vec<Triple>,
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
/// Entity object
impl Entity {
    /// Entity ID
    fn id(&self) -> &str {
        &self.id
    }

    /// Entity name (if available)
    fn name(&self) -> Option<&str> {
        self.attributes
            .iter()
            .find(|triple| triple.attribute == system_ids::NAME)
            .map(|triple| triple.value.as_str())
    }

    /// The space ID of the entity (note: the same entity can exist in multiple spaces)
    fn space_id(&self) -> &str {
        &self.space_id
    }

    fn created_at(&self) -> String {
        self.created_at.to_rfc3339()
    }

    fn created_at_block(&self) -> &str {
        &self.created_at_block
    }

    fn updated_at(&self) -> String {
        self.updated_at.to_rfc3339()
    }

    fn updated_at_block(&self) -> &str {
        &self.updated_at_block
    }

    /// Types of the entity (which are entities themselves)
    async fn types<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Entity> {
        if self.types.contains(&system_ids::RELATION_TYPE.to_string()) {
            // Since relations are also entities, and a relation's types are modelled differently
            // in Neo4j, we need to check fetch types differently if the entity is a relation.
            // mapping::Relation::<mapping::Triples>::find_types(
            //     &executor.context().0.neo4j,
            //     &self.id,
            //     &self.space_id,
            // )
            // .await
            // .expect("Failed to find relations")
            // .into_iter()
            // .map(|rel| rel.into())
            // .collect::<Vec<_>>()

            // For now, we'll just return the relation type
            mapping::Entity::<mapping::Triples>::find_by_id(
                &executor.context().0,
                system_ids::RELATION_TYPE,
                &self.space_id,
            )
            .await
            .expect("Failed to find types")
            .map(|rel| vec![rel.into()])
            .unwrap_or(vec![])
        } else {
            mapping::Entity::<mapping::Triples>::find_types(
                &executor.context().0,
                &self.id,
                &self.space_id,
            )
            .await
            .expect("Failed to find relations")
            .into_iter()
            .map(|rel| rel.into())
            .collect::<Vec<_>>()
        }
    }

    /// Attributes of the entity
    fn attributes(&self, filter: Option<AttributeFilter>) -> Vec<&Triple> {
        match filter {
            Some(AttributeFilter {
                value_type: Some(value_type),
            }) => self
                .attributes
                .iter()
                .filter(|triple| triple.value_type == value_type)
                .collect::<Vec<_>>(),
            _ => self.attributes.iter().collect::<Vec<_>>(),
        }
    }

    /// Relations outgoing from the entity
    async fn relations<'a, S: ScalarValue>(
        &'a self,
        r#where: Option<EntityRelationFilter>,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Relation> {
        mapping::Entity::<mapping::Triples>::find_relations::<mapping::Triples>(
            &executor.context().0,
            &self.id,
            r#where.map(Into::into),
        )
        .await
        .expect("Failed to find relations")
        .into_iter()
        .map(|rel| rel.into())
        .collect::<Vec<_>>()
    }
}

impl From<mapping::Entity<mapping::Triples>> for Entity {
    fn from(entity: mapping::Entity<mapping::Triples>) -> Self {
        Self {
            id: entity.attributes.id,
            types: entity.types,
            space_id: entity.attributes.system_properties.space_id.clone(),
            created_at: entity.attributes.system_properties.created_at,
            created_at_block: entity.attributes.system_properties.created_at_block,
            updated_at: entity.attributes.system_properties.updated_at,
            updated_at_block: entity.attributes.system_properties.updated_at_block,
            attributes: entity
                .attributes
                .attributes
                .into_iter()
                .map(|(key, triple)| Triple {
                    space_id: entity.attributes.system_properties.space_id.clone(),
                    attribute: key,
                    value: triple.value,
                    value_type: triple.value_type.into(),
                    options: Options {
                        format: triple.options.format,
                        unit: triple.options.unit,
                        language: triple.options.language,
                    },
                })
                .collect(),
        }
    }
}
