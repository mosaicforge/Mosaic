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

    pub async fn find_node_one<T: for<'a> Deserialize<'a>>(
        &self,
        query: neo4rs::Query,
    ) -> anyhow::Result<Option<T>> {
        Ok(self
            .neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| row.get("n"))
            .transpose()?)
    }

    pub async fn find_node_by_id<T: for<'a> Deserialize<'a>>(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<T>> {
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

    pub async fn find_relation_by_id<T: for<'a> Deserialize<'a>>(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<T>> {
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

    pub async fn find_types<T: for<'a> Deserialize<'a>>(&self) -> anyhow::Result<Vec<T>> {
        let query = neo4rs::query(&format!("MATCH (t:`{}`) RETURN t", system_ids::SCHEMA_TYPE));
        self.find_all(query).await
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
