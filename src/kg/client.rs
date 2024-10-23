use std::fmt::Display;

use futures::TryStreamExt;
use serde::Deserialize;

use crate::{
    grc20::{self},
    ops::ops::Op,
    system_ids,
};

pub fn create_geo_id() -> String {
    uuid::Uuid::new_v4().to_string().replace("-", "")
}

#[derive(Clone)]
pub struct Client {
    pub neo4j: neo4rs::Graph,
}

fn starts_with_non_alpha(s: &str) -> bool {
    s.chars().next().map_or(false, |c| !c.is_ascii_alphabetic())
}

fn contains_space(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_whitespace())
}

/// A label for a type in the KG.
/// Displays as UpperCamelCase.
pub struct TypeLabel(String);

impl TypeLabel {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl Display for TypeLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label: String = format!("{}", heck::AsUpperCamelCase(&self.0));

        if starts_with_non_alpha(&self.0) {
            write!(f, "`{}`", label)
        } else {
            write!(f, "{}", label)
        }
    }
}

/// A label for a relation in the KG.
/// Displays as SHOUTY_SNAKE_CASE.
pub struct RelationLabel(String);

impl RelationLabel {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl Display for RelationLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label: String = format!("{}", heck::AsShoutySnakeCase(&self.0));

        if starts_with_non_alpha(&self.0) {
            write!(f, "`{}`", label)
        } else {
            write!(f, "{}", label)
        }
    }
}

/// A label for an attribute in the KG.
/// Displays as lowerCamelCase.
pub struct AttributeLabel(String);

impl AttributeLabel {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl Display for AttributeLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label: String = format!("{}", heck::AsLowerCamelCase(&self.0));

        if starts_with_non_alpha(&self.0) {
            write!(f, "`{}`", label)
        } else {
            write!(f, "{}", label)
        }
    }
}

impl Client {
    pub async fn new(uri: &str, user: &str, pass: &str) -> anyhow::Result<Self> {
        let neo4j = neo4rs::Graph::new(uri, user, pass).await?;
        Ok(Self { neo4j })
    }

    // pub async fn bootstrap(&self) -> anyhow::Result<()> {
    //     let mut txn = self.neo4j.start_txn().await?;

    //     txn.run_queries([include_str!("../resources/bootstrap.cypher")])
    //         .await?;

    //     txn.commit().await?;

    //     Ok(())
    // }

    pub async fn find_one<T: for<'a> Deserialize<'a>>(
        &self,
        query: neo4rs::Query,
    ) -> anyhow::Result<Option<T>> {
        Ok(self
            .neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| row.to())
            .transpose()?)
    }

    pub async fn find_all<T: for<'a> Deserialize<'a>>(
        &self,
        query: neo4rs::Query,
    ) -> anyhow::Result<Vec<T>> {
        Ok(self
            .neo4j
            .execute(query)
            .await?
            .into_stream_as::<T>()
            .try_collect::<Vec<_>>()
            .await?)
    }

    pub async fn find_node_one<T: for<'a> Deserialize<'a>>(&self, query: neo4rs::Query) -> anyhow::Result<Option<T>> {
        Ok(self
            .neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| row.get("n"))
            .transpose()?)
    }

    pub async fn find_node_by_id<T: for<'a> Deserialize<'a>>(&self, id: &str) -> anyhow::Result<Option<T>> {
        let query = neo4rs::query("MATCH (n { id: $id }) RETURN n").param("id", id);
        self.find_node_one(query).await
    }

    pub async fn find_relation<T: for<'a> Deserialize<'a>>(
        &self,
        query: neo4rs::Query,
    ) -> anyhow::Result<Option<T>> {
        Ok(self
            .neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| row.get("r"))
            .transpose()?)
    }

    pub async fn find_relation_by_id<T: for<'a> Deserialize<'a>>(&self, id: &str) -> anyhow::Result<Option<T>> {
        let query = neo4rs::query("MATCH () -[r]-> () WHERE r.id = $id RETURN r").param("id", id);
        self.find_relation(query).await
    }

    pub async fn get_name(&self, entity_id: &str) -> anyhow::Result<Option<String>> {
        match self
            .find_one::<Entity>(Entity::find_by_id_query(&entity_id))
            .await?
        {
            Some(Entity {
                name: Some(name), ..
            }) => Ok(Some(name)),
            None | Some(Entity { name: None, .. }) => Ok(None),
        }
    }

    pub async fn set_name(&self, entity_id: &str, name: &str) -> anyhow::Result<()> {
        if name.is_empty() {
            return Ok(());
        }

        tracing::info!("SetName: {}, {}", entity_id, name);
        let mut txn = self.neo4j.start_txn().await?;

        let old_name = self
            .get_name(entity_id)
            .await?
            .unwrap_or(entity_id.to_string());

        // Rename Entity
        txn.run(
            neo4rs::query(&format!(
                r#"
                    MERGE (n {{ id: $id }})
                    ON CREATE
                        SET n.name = $value
                    ON MATCH
                        SET n.name = $value
                    "#,
            ))
            .param("id", entity_id)
            .param("value", name),
        )
        .await?;

        // Rename properties
        txn.run(neo4rs::query(&format!(
            r#"
                    MATCH (n) 
                    WHERE n.{old_attribute} IS NOT NULL
                    SET n.{new_attribute} = n.{old_attribute}
                    REMOVE n.{old_attribute}
                    "#,
            old_attribute = AttributeLabel::new(&old_name),
            new_attribute = AttributeLabel::new(name),
        )))
        .await?;

        // Rename relation
        txn.run(neo4rs::query(&format!(
            r#"
                MATCH (n) -[r:{old_relation}]-> (m)
                CREATE (n) -[:{new_relation} {{id: r.id}}]-> (m)
                DELETE r
            "#,
            old_relation = RelationLabel::new(&old_name),
            new_relation = RelationLabel::new(name)
        )))
        .await?;

        // Rename type label
        txn.run(neo4rs::query(&format!(
            r#"
                MATCH (n:{old_type})
                REMOVE n:{old_type}
                SET n:{new_type}
            "#,
            old_type = TypeLabel::new(&old_name),
            new_type = TypeLabel::new(name)
        )))
        .await?;

        Ok(txn.commit().await?)
    }

    pub async fn handle_op(&self, op: Op) -> anyhow::Result<()> {
        Ok(op.apply_op(self).await?)
    }
}

#[derive(Debug, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub cover: Option<String>,
    pub content: Option<String>,
}

impl Entity {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: Some(name.to_string()),
            description: None,
            cover: None,
            content: None,
        }
    }

    // pub fn create_query(&self) -> neo4rs::Query {
    //     let labels = self.types.iter()
    //         .map(|s| format!(":{}", s))
    //         .collect::<Vec<String>>()
    //         .join("");

    //     let query = neo4rs::query(&format!("CREATE (n{labels} {{ id: $id, name: $name }})"))
    //         .param("id", self.id.clone())
    //         .param("name", self.name.clone());

    //     query
    // }

    pub fn find_by_id_query(id: &str) -> neo4rs::Query {
        let query = neo4rs::query("MATCH (n { id: $id }) RETURN n").param("id", id);

        query
    }
}
