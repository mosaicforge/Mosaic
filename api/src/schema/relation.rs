use juniper::{graphql_object, Executor, FieldResult, ScalarValue};

use grc20_core::{
    mapping::{query_utils::Query, relation_edge, EntityNode, RelationEdge},
    neo4rs,
};

use crate::context::KnowledgeGraph;

use super::Entity;

#[derive(Debug)]
pub struct Relation {
    node: RelationEdge<EntityNode>,
    space_id: String,
    space_version: Option<String>,
    strict: bool,
}

impl Relation {
    pub fn new(
        node: RelationEdge<EntityNode>,
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

        Ok(
            relation_edge::find_one::<EntityNode>(neo4j, id, space_id.clone(), space_version.clone())
                .send()
                .await?
                .map(|node| Relation::new(node, space_id, space_version, strict)),
        )
    }
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
/// Relation object
impl Relation {
    /// Relation ID
    fn id(&self) -> &str {
        &self.node.id
    }

    /// Entity of the relation
    async fn entity<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Entity> {
        Ok(Entity::load(
            &executor.context().neo4j,
            &self.node.id,
            self.space_id.clone(),
            self.space_version.clone(),
            self.strict,
        )
        .await?
        .expect("Relation entity not found"))
    }

    /// Relation type of the relation
    async fn relation_type<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> FieldResult<Entity> {
        Ok(Entity::load(
            &executor.context().neo4j,
            &self.node.relation_type,
            self.space_id.clone(),
            self.space_version.clone(),
            self.strict,
        )
        .await?
        .expect("Relation type entity not found"))
    }

    /// Entity from which the relation originates
    async fn from(
        &self,
    ) -> FieldResult<Entity> {
        Ok(Entity::new(
            self.node.from.clone(),
            self.space_id.clone(),
            self.space_version.clone(),
            self.strict,
        ))
    }

    /// Entity to which the relation points
    async fn to(
        &self,
    ) -> FieldResult<Entity> {
        Ok(Entity::new(
            self.node.to.clone(),
            self.space_id.clone(),
            self.space_version.clone(),
            self.strict,
        ))
    }
}
