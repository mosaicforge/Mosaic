use juniper::{graphql_object, Executor, ScalarValue};

use sdk::{
    mapping::{query_utils::Query, relation_node, RelationNode},
    neo4rs,
};

use crate::context::KnowledgeGraph;

use super::Entity;

#[derive(Debug)]
pub struct Relation {
    node: RelationNode,
    space_id: String,
    space_version: Option<String>,
}

impl Relation {
    pub fn new(node: RelationNode, space_id: String, space_version: Option<String>) -> Self {
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
        space_version: Option<String>,
    ) -> Option<Self> {
        let id = id.into();
        let space_id = space_id.into();

        relation_node::find_one(neo4j, id, space_id.clone(), space_version.clone())
            .send()
            .await
            .expect("Failed to find relation")
            .map(|node| Relation::new(node, space_id, space_version))
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
    ) -> Entity {
        Entity::load(
            &executor.context().0,
            &self.node.id,
            self.space_id.clone(),
            self.space_version.clone(),
        )
        .await
        .expect("Relation entity not found")
    }

    /// Relation type of the relation
    async fn relation_type<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Entity {
        Entity::load(
            &executor.context().0,
            &self.node.relation_type,
            self.space_id.clone(),
            self.space_version.clone(),
        )
        .await
        .expect("Relation type entity not found")
    }

    /// Entity from which the relation originates
    async fn from<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Entity {
        Entity::load(
            &executor.context().0,
            &self.node.from,
            self.space_id.clone(),
            self.space_version.clone(),
        )
        .await
        .expect("From entity not found")
    }

    /// Entity to which the relation points
    async fn to<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Entity {
        Entity::load(
            &executor.context().0,
            &self.node.to,
            self.space_id.clone(),
            self.space_version.clone(),
        )
        .await
        .expect("To entity not found")
    }
}
