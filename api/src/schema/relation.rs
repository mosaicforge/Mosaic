use chrono::{DateTime, Utc};
use juniper::{graphql_object, Executor, ScalarValue};

use sdk::{mapping, system_ids};

use crate::context::KnowledgeGraph;

use super::{Entity, EntityRelationFilter, Options, Triple};

#[derive(Debug)]
pub struct Relation {
    id: String,
    relation_type: String,
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
            .find(|triple| triple.attribute == system_ids::NAME_ATTRIBUTE)
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

    /// Relation type of the relation
    async fn relation_type<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Vec<Entity> {
        mapping::Entity::<mapping::Triples>::find_by_id(
            &executor.context().0,
            &self.relation_type,
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
        space_id: String,
        r#where: Option<EntityRelationFilter>,
    ) -> Vec<Relation> {
        let mut base_query = mapping::relation_queries::FindMany::new("r")
            .space_id(&space_id)
            .from(|from_query| from_query.id(self.id()));
        
        if let Some(filter) = r#where {
            if let Some(id) = filter.id {
                base_query = base_query.id(&id);
            }

            if let Some(to_id) = filter.to_id {
                base_query = base_query
                    .to(|to_query| to_query.id(&to_id));
            }

            if let Some(relation_type) = filter.relation_type {
                base_query = base_query
                    .relation_type(&relation_type);
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

impl From<mapping::Relation<mapping::Triples>> for Relation {
    fn from(relation: mapping::Relation<mapping::Triples>) -> Self {
        Self {
            id: relation.id().to_string(),
            relation_type: relation.r#type.clone(),
            space_id: relation.system_properties().space_id.clone(),
            created_at: relation.system_properties().created_at,
            created_at_block: relation.system_properties().created_at_block.clone(),
            updated_at: relation.system_properties().updated_at,
            updated_at_block: relation.system_properties().updated_at_block.clone(),
            attributes: relation
                .entity
                .attributes
                .attributes
                .iter()
                .map(|(key, triple)| Triple {
                    space_id: relation.system_properties().space_id.clone(),
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
