use chrono::{DateTime, Utc};
use juniper::{graphql_object, Executor, ScalarValue};

use sdk::{mapping, system_ids};

use crate::context::KnowledgeGraph;

use super::{Entity, Options, Triple};

#[derive(Debug)]
pub struct Relation {
    id: String,
    relation_types: Vec<String>,
    space_id: String,
    created_at: DateTime<Utc>,
    created_at_block: String,
    updated_at: DateTime<Utc>,
    updated_at_block: String,
    attributes: Vec<Triple>,
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
/// Relation object
///
/// Note: Relations are also entities, but they have a different structure in the database.
/// In other words, the Relation object is a "view" on a relation entity. All relations
/// can also be queried as entities.
impl Relation {
    /// Relation ID
    fn id(&self) -> &str {
        &self.id
    }

    /// Relation name (if available)
    fn name(&self) -> Option<&str> {
        self.attributes
            .iter()
            .find(|triple| triple.attribute == system_ids::NAME)
            .map(|triple| triple.value.as_str())
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

    /// Attributes of the relation
    fn attributes(&self) -> &[Triple] {
        &self.attributes
    }

    /// Relation types of the relation
    async fn relation_types<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Entity> {
        mapping::Entity::<mapping::Triples>::find_by_ids(
            &executor.context().0,
            &self.relation_types,
            &self.space_id,
        )
        .await
        .expect("Failed to find types")
        .into_iter()
        .filter(|rel| rel.id() != system_ids::RELATION_TYPE)
        .map(|rel| rel.into())
        .collect::<Vec<_>>()
    }

    /// Entity from which the relation originates
    async fn from<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Entity {
        mapping::Relation::<mapping::Triples>::find_from::<mapping::Triples>(
            &executor.context().0,
            &self.id,
            &self.space_id,
        )
        .await
        .expect("Failed to find node")
        .map(Entity::from)
        .unwrap()
    }

    /// Entity to which the relation points
    async fn to<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Entity {
        mapping::Relation::<mapping::Triples>::find_to::<mapping::Triples>(
            &executor.context().0,
            &self.id,
            &self.space_id,
        )
        .await
        .expect("Failed to find node")
        .map(Entity::from)
        .unwrap()
    }

    /// Relations outgoing from the relation
    async fn relations<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Relation> {
        mapping::Entity::<mapping::Triples>::find_relations::<mapping::Triples>(
            &executor.context().0,
            &self.id,
            Some(mapping::EntityRelationFilter {
                space_id: Some(self.space_id.clone()),
                ..Default::default()
            }),
        )
        .await
        .expect("Failed to find relations")
        .into_iter()
        .map(|rel| rel.into())
        .collect::<Vec<_>>()
    }
}

impl From<mapping::Relation<mapping::Triples>> for Relation {
    fn from(relation: mapping::Relation<mapping::Triples>) -> Self {
        Self {
            id: relation.attributes.id,
            relation_types: relation.types,
            space_id: relation.attributes.system_properties.space_id.clone(),
            created_at: relation.attributes.system_properties.created_at,
            created_at_block: relation
                .attributes
                .system_properties
                .created_at_block
                .clone(),
            updated_at: relation.attributes.system_properties.updated_at,
            updated_at_block: relation
                .attributes
                .system_properties
                .updated_at_block
                .clone(),
            attributes: relation
                .attributes
                .attributes
                .iter()
                .map(|(key, triple)| Triple {
                    // entiti: triple.entity,
                    space_id: relation.attributes.system_properties.space_id.clone(),
                    attribute: key.to_string(),
                    value: triple.value.clone(),
                    value_type: triple.value_type.clone().into(),
                    options: Options {
                        format: triple.options.format.clone(),
                        unit: triple.options.unit.clone(),
                        language: triple.options.language.clone(),
                    },
                })
                .collect(),
        }
    }
}
