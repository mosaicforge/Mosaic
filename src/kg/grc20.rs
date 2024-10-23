use std::fmt::Display;

use crate::system_ids;

use super::Client;

pub struct Entity {
    kg: Client,
    pub id: String,
    pub name: String,
}

impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}

impl Entity {
    pub fn from_entity(kg: Client, entity_node: EntityNode) -> Self {
        Self { kg, id: entity_node.id.clone(), name: entity_node.name.unwrap_or(entity_node.id) }
    }

    pub async fn value_type(&self) -> anyhow::Result<Option<Self>> {
        let query = neo4rs::query(&format!(
            r#"
            MATCH (a {{id: $id}}) -[:`{value_type_attr}`]-> (t:`{type_type}`)
            WHERE t.id IS NOT NULL AND t.`{name_attr}` IS NOT NULL
            RETURN t
            "#,
            value_type_attr = system_ids::VALUE_TYPE,
            type_type = system_ids::SCHEMA_TYPE,
            name_attr = system_ids::NAME,
        ))
        .param("id", self.id.clone());

        let type_node = self.kg.find_one::<EntityNode>(query).await?;

        Ok(type_node
            .map(|node| {
                Self::from_entity(self.kg.clone(), node)
            })
        )
    }

    pub async fn attributes(&self) -> anyhow::Result<Vec<Self>> {
        let query = neo4rs::query(&format!(
            r#"
            MATCH ({{id: $id}}) -[:`{attr_attr}`]-> (a:`{attr_type}`)
            WHERE a.id IS NOT NULL AND a.`{name_attr}` IS NOT NULL
            RETURN a
            "#,
            attr_attr = system_ids::ATTRIBUTES,
            attr_type = system_ids::ATTRIBUTE,
            name_attr = system_ids::NAME,
        ))
        .param("id", self.id.clone());

        let attribute_nodes = self.kg.find_all::<EntityNode>(query).await?;

        Ok(attribute_nodes
            .into_iter()
            .map(|node| {
                Entity::from_entity(self.kg.clone(), node)
            })
            .collect::<Vec<_>>()
        )
    }
}

#[derive(serde::Deserialize)]
pub struct EntityNode {
    id: String,
    #[serde(default, rename = "a126ca530c8e48d5b88882c734c38935")]  // TODO: Find a way to use system_ids constants
    name: Option<String>,
}
