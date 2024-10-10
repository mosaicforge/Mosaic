use futures::TryStreamExt;
use serde::Deserialize;

use crate::grc20::{self};

pub struct KgClient {
    neo4j: neo4rs::Graph,
}

// pub fn label_name(name: &str) -> String {

// }

impl KgClient {
    pub async fn new(uri: &str, user: &str, pass: &str) -> anyhow::Result<Self> {
        let neo4j = neo4rs::Graph::new(uri, user, pass).await?;
        Ok(Self { neo4j })
    }

    pub async fn bootstrap(&self) -> anyhow::Result<()> {
        let mut txn = self.neo4j.start_txn().await?;

        txn.run_queries([include_str!("../resources/bootstrap.cypher")])
            .await?;

        txn.commit().await?;

        Ok(())
    }

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

        println!("SetName: {}, {}", entity_id, name);
        let mut txn = self.neo4j.start_txn().await?;

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

        // // Rename properties
        // txn.run(
        //     neo4rs::query(&format!(
        //         r#"
        //             MATCH (n) 
        //             WHERE n._{entity_id} IS NOT NULL
        //             SET n.{property} = n._{entity_id}
        //             REMOVE n._{entity_id}
        //             "#,
        //         entity_id = entity_id,
        //         property = heck::AsLowerCamelCase(name),
        //     )),
        // )
        // .await?;

        // // Rename relation
        // txn.run(neo4rs::query(&format!(
        //     r#"
        //         MATCH (n) -[r:_{entity_id}]-> (m)
        //         CREATE (n) -[:{relation_label}]-> (m)
        //         DELETE r
        //     "#,
        //     entity_id = entity_id,
        //     relation_label = heck::AsShoutySnakeCase(name)
        // )))
        // .await?;

        // // Rename labels
        // txn.run(neo4rs::query(&format!(
        //     r#"
        //         MATCH (n) 
        //         WHERE n:_{entity_id}
        //         REMOVE n:_{entity_id}
        //         SET n:{label}
        //     "#,
        //     entity_id = entity_id,
        //     label = heck::AsUpperCamelCase(name)
        // )))
        // .await?;

        Ok(txn.commit().await?)
    }

    pub async fn set_triple(
        &self,
        entity_id: &str,
        attribute_id: &str,
        value: grc20::Value,
    ) -> anyhow::Result<()> {
        let entity_name = self
            .find_one::<Entity>(Entity::find_by_id_query(&entity_id))
            .await?
            .and_then(|entity| entity.name)
            .unwrap_or(entity_id.to_string());

        let attribute_label = self
            .get_name(attribute_id)
            .await?
            .unwrap_or(attribute_id.to_string());

        println!(
            "SetTriple: {}, {}, {:?}",
            entity_name, attribute_label, value,
        );

        match (value.r#type(), attribute_label.as_str()) {
            (grc20::ValueType::Entity, "Type") => {
                let type_label = self
                    .get_name(&value.value)
                    .await?
                    .map_or(format!("_{}", value.value), |name| {
                        format!("{}", heck::AsUpperCamelCase(name))
                    });

                self.neo4j
                    .run(
                        neo4rs::query(&format!(
                            r#"
                            MERGE (n {{ id: $id }}) 
                            ON CREATE 
                                SET n :{type_label} 
                            ON MATCH 
                                SET n :{type_label}
                            "#,
                        ))
                        .param("id", entity_id),
                    )
                    .await?;
            }
            // (grc20::ValueType::Text, "Name") => {
            //     self.set_name(entity_id, &value.value).await?;
            // }
            (grc20::ValueType::Entity, label) => {
                let relation_label = self
                    .get_name(attribute_id)
                    .await?
                    .map_or(format!("_{}", attribute_id), |name| {
                        format!("{}", heck::AsShoutySnakeCase(name))
                    });

                self.neo4j
                    .run(
                        neo4rs::query(&format!(
                            r#"
                            MERGE (n {{ id: $id }})
                            MERGE (m {{ id: $value }})
                            MERGE (n) -[:{relation_label}]-> (m)
                            "#,
                        ))
                        .param("id", entity_id)
                        .param("value", value.value),
                    )
                    .await?;
            }
            (_, label) => {
                let attribute_name = self
                    .get_name(attribute_id)
                    .await?
                    .map_or(format!("_{}", attribute_id), |name| {
                        format!("{}", heck::AsLowerCamelCase(name))
                    });

                self.neo4j
                    .run(
                        neo4rs::query(&format!(
                            r#"
                            MERGE (n {{ id: $id }}) 
                            ON CREATE
                                SET n.{attribute_name} = $value
                            ON MATCH
                                SET n.{attribute_name} = $value
                            "#,
                        ))
                        .param("id", entity_id)
                        .param("value", value.value),
                    )
                    .await?;
            }
        };

        Ok(())
    }

    pub async fn delete_tripe(
        &self,
        entity_id: &str,
        attribute_id: &str,
        value: grc20::Value,
    ) -> anyhow::Result<()> {
        let entity_name = self
            .find_one::<Entity>(Entity::find_by_id_query(&entity_id))
            .await?
            .and_then(|entity| entity.name)
            .unwrap_or(entity_id.to_string());

        let attribute_label = self
            .get_name(attribute_id)
            .await?
            .unwrap_or(attribute_id.to_string());

        println!(
            "DeleteTriple: {}, {}, {:?}",
            entity_name, attribute_label, value,
        );

        match (value.r#type(), attribute_label.as_str()) {
            (grc20::ValueType::Entity, "Type") => {
                let type_label = self
                    .get_name(&value.value)
                    .await?
                    .map_or(format!("_{}", value.value), |name| {
                        format!("{}", heck::AsUpperCamelCase(name))
                    });

                self.neo4j
                    .run(
                        neo4rs::query(&format!(
                            r#"
                            MATCH (n {{ id: $id }})
                            REMOVE n :{type_label}
                            "#,
                        ))
                        .param("id", entity_id),
                    )
                    .await?;
            }
            (grc20::ValueType::Entity, label) => {
                let relation_label = self
                    .get_name(attribute_id)
                    .await?
                    .map_or(format!("_{}", attribute_id), |name| {
                        format!("{}", heck::AsShoutySnakeCase(name))
                    });

                self.neo4j
                    .run(
                        neo4rs::query(&format!(
                            r#"
                            MATCH ({{ id: $id }}) -[r:{relation_label}]-> ({{ id: $value }})
                            DELETE r
                            "#,
                        ))
                        .param("id", entity_id)
                        .param("value", value.value),
                    )
                    .await?;
            }
            (_, label) => {
                let attribute_name = self
                    .get_name(attribute_id)
                    .await?
                    .map_or(format!("_{}", attribute_id), |name| {
                        format!("{}", heck::AsLowerCamelCase(name))
                    });

                self.neo4j
                    .run(
                        neo4rs::query(&format!(
                            r#"
                            MATCH (n {{ id: $id }}) 
                            REMOVE n.{attribute_name}
                            "#,
                        ))
                        .param("id", entity_id)
                        .param("value", value.value),
                    )
                    .await?;
            }
        };

        Ok(())
    }

    pub async fn handle_op(&self, op: grc20::Op) -> anyhow::Result<()> {
        match (op.r#type(), op.triple) {
            (
                grc20::OpType::SetTriple,
                Some(grc20::Triple {
                    entity: entity_id,
                    attribute: attribute_id,
                    value: Some(value),
                }),
            ) => {
                self.set_triple(&entity_id, &attribute_id, value).await?;
            }
            (
                grc20::OpType::DeleteTriple,
                Some(grc20::Triple {
                    entity: entity_id,
                    attribute: attribute_id,
                    value: Some(value),
                }),
            ) => self.delete_tripe(&entity_id, &attribute_id, value).await?,
            _ => (),
        };

        Ok(())
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
