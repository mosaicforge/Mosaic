use chrono::{DateTime, Utc};
use futures::stream::TryStreamExt;

use serde::{Deserialize, Serialize};

use crate::{error::DatabaseError, indexer_ids, models::BlockMetadata};

use super::{
    attributes,
    query_utils::{AttributeFilter, PropFilter, Query, QueryPart},
    triple, AttributeNode, Triple,
};

/// Neo4j model of an Entity
#[derive(Debug, Deserialize, PartialEq)]
pub struct EntityNode {
    pub id: String,

    /// System properties
    #[serde(flatten)]
    pub system_properties: SystemProperties,
}

impl EntityNode {
    pub fn get_attributes(
        &self,
        neo4j: &neo4rs::Graph,
        space_id: &str,
        space_version: Option<i64>,
    ) -> attributes::FindOneQuery {
        attributes::FindOneQuery::new(neo4j, self.id.clone(), space_id.to_owned(), space_version)
    }

    pub fn set_attribute(
        &self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: &str,
        space_version: i64,
        attribute: AttributeNode,
    ) -> triple::InsertOneQuery {
        triple::InsertOneQuery::new(
            neo4j,
            block,
            space_id.to_owned(),
            space_version,
            Triple {
                entity: self.id.clone(),
                attribute: attribute.id,
                value: attribute.value,
            },
        )
    }

    pub fn set_attributes<T>(
        &self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: &str,
        space_version: i64,
        attributes: T,
    ) -> attributes::InsertOneQuery<T> {
        attributes::InsertOneQuery::new(
            neo4j,
            block,
            self.id.clone(),
            space_id.to_owned(),
            space_version,
            attributes,
        )
    }

    pub fn delete(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: impl Into<String>,
        space_version: i64,
    ) -> DeleteOneQuery {
        DeleteOneQuery::new(neo4j, block, self.id, space_id.into(), space_version)
    }
}

pub fn delete_one(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: i64,
) -> DeleteOneQuery {
    DeleteOneQuery::new(
        neo4j,
        block,
        entity_id.into(),
        space_id.into(),
        space_version,
    )
}

pub fn find_one(neo4j: &neo4rs::Graph, id: impl Into<String>) -> FindOneQuery {
    FindOneQuery::new(neo4j, id.into())
}

pub fn find_many(neo4j: &neo4rs::Graph) -> FindManyQuery {
    FindManyQuery::new(neo4j)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SystemProperties {
    #[serde(rename = "82nP7aFmHJLbaPFszj2nbx")] // CREATED_AT_TIMESTAMP
    pub created_at: DateTime<Utc>,
    #[serde(rename = "59HTYnd2e4gBx2aA98JfNx")] // CREATED_AT_BLOCK
    pub created_at_block: String,
    #[serde(rename = "5Ms1pYq8v8G1RXC3wWb9ix")] // UPDATED_AT_TIMESTAMP
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "7pXCVQDV9C7ozrXkpVg8RJ")] // UPDATED_AT_BLOCK
    pub updated_at_block: String,
}

impl Default for SystemProperties {
    fn default() -> Self {
        Self {
            created_at: Default::default(),
            created_at_block: "0".to_string(),
            updated_at: Default::default(),
            updated_at_block: "0".to_string(),
        }
    }
}

pub struct FindOneQuery {
    neo4j: neo4rs::Graph,
    id: String,
}

impl FindOneQuery {
    pub fn new(neo4j: &neo4rs::Graph, id: String) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id,
        }
    }
}

impl Query<Option<EntityNode>> for FindOneQuery {
    async fn send(self) -> Result<Option<EntityNode>, DatabaseError> {
        const QUERY: &str = r#"
            MATCH (n {id: $id})
            RETURN n
        "#;

        let query = neo4rs::query(QUERY).param("id", self.id);

        #[derive(Debug, Deserialize)]
        struct RowResult {
            n: EntityNode,
        }

        Ok(self
            .neo4j
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                Result::<_, DatabaseError>::Ok(row.n)
            })
            .transpose()?)
    }
}

pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    id: Option<PropFilter<String>>,
    attributes: Vec<AttributeFilter>,
}

impl FindManyQuery {
    pub fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id: None,
            attributes: Vec::new(),
        }
    }

    pub fn id(mut self, id: PropFilter<String>) -> Self {
        self.id = Some(id);
        self
    }

    pub fn attribute(mut self, attribute: AttributeFilter) -> Self {
        self.attributes.push(attribute);
        self
    }

    fn into_query_part(self) -> QueryPart {
        let mut query_part = QueryPart::default().match_clause("(e)").return_clause("e");

        if let Some(id) = self.id {
            query_part.merge_mut(id.into_query_part("e", "id"));
        }

        for attribute in self.attributes {
            query_part.merge_mut(attribute.into_query_part("e"));
        }

        query_part
    }
}

impl Query<Vec<EntityNode>> for FindManyQuery {
    async fn send(self) -> Result<Vec<EntityNode>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query = self.into_query_part().build();

        #[derive(Debug, Deserialize)]
        struct RowResult {
            e: EntityNode,
        }

        Ok(neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move { Ok(row.e) })
            .try_collect::<Vec<_>>()
            .await?)
    }
}

pub struct DeleteOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    id: String,
    space_id: String,
    space_version: i64,
}

impl DeleteOneQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        id: String,
        space_id: String,
        space_version: i64,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            id,
            space_id,
            space_version,
        }
    }
}

impl Query<()> for DeleteOneQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (e {{id: $entity_id}}) -[r:ATTRIBUTE {{space_id: $space_id, max_version: null}}]-> ()
            SET r.max_version = $space_version
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            "#,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("entity_id", self.id)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("block_timestamp", self.block.timestamp.to_rfc3339())
            .param("block_number", self.block.block_number.to_string());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use testcontainers::{
        core::{IntoContainerPort, WaitFor},
        runners::AsyncRunner,
        GenericImage, ImageExt,
    };

    const BOLT_PORT: u16 = 7687;
    const HTTP_PORT: u16 = 7474;

    #[tokio::test]
    async fn test_find_by_id() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let container = GenericImage::new("neo4j", "2025.01.0-community")
            .with_wait_for(WaitFor::Duration {
                length: std::time::Duration::from_secs(5),
            })
            .with_exposed_port(BOLT_PORT.tcp())
            .with_exposed_port(HTTP_PORT.tcp())
            .with_env_var("NEO4J_AUTH", "none")
            .start()
            .await
            .expect("Failed to start Neo 4J container");

        let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
        let host = container.get_host().await.unwrap().to_string();

        let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
            .await
            .unwrap();

        let triple = Triple {
            entity: "abc".to_string(),
            attribute: "name".to_string(),
            value: "Alice".into(),
        };

        triple
            .insert(&neo4j, &BlockMetadata::default(), "space_id", 0)
            .send()
            .await
            .expect("Failed to insert triple");

        let entity = find_one(&neo4j, "abc")
            .send()
            .await
            .expect("Failed to find entity")
            .expect("Entity not found");

        assert_eq!(
            entity,
            EntityNode {
                id: "abc".to_string(),
                system_properties: SystemProperties::default(),
            }
        );
    }
}
