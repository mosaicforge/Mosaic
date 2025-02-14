use std::collections::HashMap;

use futures::TryStreamExt;
use neo4rs::BoltType;
use serde::Deserialize;

use crate::{error::DatabaseError, indexer_ids, models::BlockMetadata};

use super::{
    query_utils::{PropFilter, Query, QueryPart, VersionFilter},
    Options, Value, ValueType,
};

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct Triple {
    pub entity: String,
    pub attribute: String,

    #[serde(flatten)]
    pub value: Value,
}

impl Triple {
    pub fn new(
        entity: String,
        attribute: String,
        value: String,
        value_type: ValueType,
        options: Options,
    ) -> Self {
        Self {
            entity,
            attribute,
            value: Value {
                value,
                value_type,
                options,
            },
        }
    }

    pub fn insert(
        self,
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: i64,
    ) -> InsertOneQuery {
        InsertOneQuery::new(neo4j, block, space_id, space_version, self)
    }
}

pub fn insert_many(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: String,
    space_version: i64,
    triples: Vec<Triple>,
) -> InsertManyQuery {
    InsertManyQuery::new(neo4j, block, space_id, space_version, triples)
}

pub fn find_one(
    neo4j: &neo4rs::Graph,
    attribute_id: String,
    entity_id: String,
    space_id: String,
    space_version: Option<i64>,
) -> FindOneQuery {
    FindOneQuery::new(neo4j, attribute_id, entity_id, space_id, space_version)
}

pub fn find_many<T>(neo4j: &neo4rs::Graph) -> FindManyQuery<T> {
    FindManyQuery::new(neo4j)
}

impl Into<BoltType> for Triple {
    fn into(self) -> BoltType {
        let mut triple_bolt_map = HashMap::new();
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "entity".into(),
            },
            self.entity.into(),
        );
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "attribute".into(),
            },
            self.attribute.into(),
        );
        triple_bolt_map.insert(
            neo4rs::BoltString {
                value: "value".into(),
            },
            self.value.into(),
        );

        BoltType::Map(neo4rs::BoltMap {
            value: triple_bolt_map,
        })
    }
}

pub struct InsertOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: i64,
    triple: Triple,
}

impl InsertOneQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: i64,
        triple: Triple,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            triple,
        }
    }
}

impl Query<()> for InsertOneQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MERGE (e {{id: $triple.entity}})
            ON CREATE SET e += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number
            }}
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            WITH e
            CALL (e) {{
                MATCH (e) -[r:ATTRIBUTE {{space_id: $space_id}}]-> ({{id: $triple.attribute}})
                WHERE r.max_version IS null
                SET r.max_version = $space_version
            }}
            CALL (e) {{
                CREATE (e) -[:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (m)
                SET m = $triple.value
                SET m.id = $triple.attribute
            }}
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("triple", self.triple)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

pub struct InsertManyQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: i64,
    triples: Vec<Triple>,
}

impl InsertManyQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: i64,
        triples: Vec<Triple>,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            triples,
        }
    }
}

impl Query<()> for InsertManyQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            UNWIND $triples as triple
            MERGE (e {{id: triple.entity}})
            ON CREATE SET e += {{
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number
            }}
            SET e += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            WITH e
            CALL (e, triple) {{
                MATCH (e) -[r:ATTRIBUTE {{space_id: $space_id}}]-> ({{id: triple.attribute}})
                WHERE r.max_version IS null
                SET r.max_version = $space_version
            }}
            CALL {{
                CREATE (e) -[:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (m)
                SET m = triple.value
                SET m.id = triple.attribute
            }}
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("triples", self.triples)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

pub struct FindOneQuery {
    neo4j: neo4rs::Graph,
    attribute_id: String,
    entity_id: String,
    space_id: String,
    space_version: VersionFilter,
}

impl FindOneQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        attribute_id: String,
        entity_id: String,
        space_id: String,
        space_version: Option<i64>,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            attribute_id,
            entity_id,
            space_id,
            space_version: VersionFilter::new(space_version),
        }
    }
}

impl Query<Option<Triple>> for FindOneQuery {
    async fn send(self) -> Result<Option<Triple>, DatabaseError> {
        let query = QueryPart::default()
            .match_clause("(e {{id: $entity_id}}) -[r:ATTRIBUTE {{space_id: $space_id}}]-> (n {{attribute: $attribute_id}})")
            .merge(self.space_version.into_query_part("r"))
            .return_clause("n{{.*, entity: e.id}}")
            .params("attribute_id", self.attribute_id)
            .params("entity_id", self.entity_id)
            .params("space_id", self.space_id)
            .build();

        #[derive(Debug, Deserialize)]
        struct RowResult {
            n: Triple,
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

pub struct FindManyQuery<T> {
    neo4j: neo4rs::Graph,
    attribute_id: Option<PropFilter<String>>,
    value: Option<PropFilter<String>>,
    value_type: Option<PropFilter<String>>,
    entity_id: Option<PropFilter<String>>,
    space_id: Option<PropFilter<String>>,
    space_version: VersionFilter,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> FindManyQuery<T> {
    pub fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            attribute_id: None,
            value: None,
            value_type: None,
            entity_id: None,
            space_id: None,
            space_version: VersionFilter::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn attribute_id(mut self, filter: PropFilter<String>) -> Self {
        self.attribute_id = Some(filter);
        self
    }

    pub fn value(mut self, filter: PropFilter<String>) -> Self {
        self.value = Some(filter);
        self
    }

    pub fn value_type(mut self, filter: PropFilter<String>) -> Self {
        self.value_type = Some(filter);
        self
    }

    pub fn entity_id(mut self, filter: PropFilter<String>) -> Self {
        self.entity_id = Some(filter);
        self
    }

    pub fn space_id(mut self, filter: PropFilter<String>) -> Self {
        self.space_id = Some(filter);
        self
    }

    pub fn space_version(mut self, space_version: i64) -> Self {
        self.space_version.version_mut(space_version);
        self
    }

    fn to_query_part(self) -> QueryPart {
        let mut query_part = QueryPart::default().match_clause("(e) -[r:ATTRIBUTE]-> (n)");

        if let Some(attribute_id) = self.attribute_id {
            query_part = query_part.merge(attribute_id.into_query_part("n", "attribute"));
        }

        if let Some(value) = self.value {
            query_part = query_part.merge(value.into_query_part("n", "value"));
        }

        if let Some(value_type) = self.value_type {
            query_part = query_part.merge(value_type.into_query_part("n", "value_type"));
        }

        if let Some(entity_id) = self.entity_id {
            query_part = query_part.merge(entity_id.into_query_part("e", "id"));
        }

        if let Some(space_id) = self.space_id {
            query_part = query_part.merge(space_id.into_query_part("r", "space_id"));
        }

        query_part
            .merge(self.space_version.into_query_part("r"))
            .return_clause("n{{.*, entity: e.id}}")
    }
}

impl Query<Vec<Triple>> for FindManyQuery<Vec<Triple>> {
    async fn send(self) -> Result<Vec<Triple>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query_part = self.to_query_part();
        println!("{:?}", query_part.query());
        let query = query_part.build();

        #[derive(Debug, Deserialize)]
        struct RowResult {
            n: Triple,
        }

        neo4j
            .execute(query)
            .await?
            .into_stream_as::<RowResult>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move { Ok(row.n) })
            .try_collect::<Vec<Triple>>()
            .await
    }
}

// impl<T> Query<T> for FindManyQuery<T>
// where
//     T: FromAttributes,
// {
//     async fn send(self) -> Result<T, DatabaseError> {
//         let neo4j = self.neo4j.clone();
//         let query_part = self.to_query_part();
//         println!("{:?}", query_part.query());
//         let query = query_part.build();

//         #[derive(Debug, Deserialize)]
//         struct RowResult {
//             n: Triple,
//         }

//         let triples = neo4j
//             .execute(query)
//             .await?
//             .into_stream_as::<RowResult>()
//             .map_err(DatabaseError::from)
//             .and_then(|row| async move { Ok(row.n) })
//             .try_collect::<Vec<Triple>>()
//             .await?;

//         T::from_attributes(triples.into()).map_err(DatabaseError::TripleError)
//     }
// }

pub struct DeleteOneQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    attribute_id: String,
    entity_id: String,
    space_id: String,
    space_version: i64,
}

impl DeleteOneQuery {
    pub fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        attribute_id: String,
        entity_id: String,
        space_id: String,
        space_version: i64,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            attribute_id,
            entity_id,
            space_id,
            space_version,
        }
    }
}

impl Query<()> for DeleteOneQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (e {{id: $entity_id}}) -[r:ATTRIBUTE {{space_id: $space_id, max_version: null}}]-> ({{attribute: $attribute_id}})
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
            .param("attribute_id", self.attribute_id)
            .param("entity_id", self.entity_id)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version);

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
    async fn test_insert_find_one() {
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
            value: Value {
                value: "Alice".to_string(),
                value_type: ValueType::Text,
                options: Options::default(),
            },
        };

        triple
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "space_id".to_string(), 0)
            .send()
            .await
            .expect("Failed to insert triple");

        // let result = neo4j.execute(
        //     neo4rs::query("MATCH ({id: $entity_id}) -[r:ATTRIBUTE {space_id: $space_id}]-> (n {attribute: $attribute_id})
        //     RETURN n")
        //         .param("attribute_id", "name")
        //         .param("entity_id", "entity_id")
        //         .param("space_id", "space_id")
        //         .param("space_version", 0)
        // ).await.expect("Failed to execute query").next().await.expect("Failed to get result");
        // println!("{:?}", result);

        let found_triple = FindOneQuery::new(
            &neo4j,
            "name".to_string(),
            "entity_id".to_string(),
            "space_id".to_string(),
            Some(0),
        )
        .send()
        .await
        .expect("Failed to find triple")
        .expect("Triple not found");

        assert_eq!(triple, found_triple);
    }

    #[tokio::test]
    pub async fn test_insert_many() {
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
            value: Value {
                value: "Alice".to_string(),
                value_type: ValueType::Text,
                options: Options::default(),
            },
        };

        let other_triple = Triple {
            entity: "def".to_string(),
            attribute: "name".to_string(),
            value: Value {
                value: "Bob".to_string(),
                value_type: ValueType::Text,
                options: Options::default(),
            },
        };

        InsertManyQuery::new(
            &neo4j,
            &BlockMetadata::default(),
            "space_id".to_string(),
            0,
            vec![triple.clone(), other_triple],
        )
        .send()
        .await
        .expect("Failed to insert triples");

        let found_triples = FindManyQuery::<Vec<Triple>>::new(&neo4j)
            .attribute_id(PropFilter::new().value("name"))
            .value(PropFilter::new().value("Alice"))
            .value_type(PropFilter::new().value("TEXT"))
            .entity_id(PropFilter::new().value("entity_id"))
            .space_id(PropFilter::new().value("space_id"))
            .space_version(0)
            .send()
            .await
            .expect("Failed to find triples");

        assert_eq!(vec![triple], found_triples);
    }

    #[tokio::test]
    pub async fn test_insert_find_many() {
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
            value: Value {
                value: "Alice".to_string(),
                value_type: ValueType::Text,
                options: Options::default(),
            },
        };

        triple
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "space_id".to_string(), 0)
            .send()
            .await
            .expect("Failed to insert triple");

        let other_triple = Triple {
            entity: "def".to_string(),
            attribute: "name".to_string(),
            value: Value {
                value: "Bob".to_string(),
                value_type: ValueType::Text,
                options: Options::default(),
            },
        };

        other_triple
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "space_id".to_string(), 0)
            .send()
            .await
            .expect("Failed to insert triple");

        let found_triples = FindManyQuery::<Vec<Triple>>::new(&neo4j)
            .attribute_id(PropFilter::new().value("name"))
            .value(PropFilter::new().value("Alice"))
            .value_type(PropFilter::new().value("TEXT"))
            .entity_id(PropFilter::new().value("entity_id"))
            .space_id(PropFilter::new().value("space_id"))
            .space_version(0)
            .send()
            .await
            .expect("Failed to find triples");

        assert_eq!(vec![triple], found_triples);
    }
}
