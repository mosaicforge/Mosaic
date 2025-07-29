use super::models::Entity;
use crate::error::DatabaseError;
use crate::mapping::value::models::Value;
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct EntityQueryResult {
    entity_id: Uuid,
    spaces: Vec<Uuid>,
    properties: Vec<HashMap<Uuid, String>>,
}

#[derive(Clone)]
pub struct FindOneQuery {
    pub neo4j: neo4rs::Graph,
    pub entity_id: Uuid,
}

impl FindOneQuery {
    pub fn new(neo4j: &neo4rs::Graph, entity_id: Uuid) -> Self {
        Self {
            neo4j: neo4j.clone(),
            entity_id,
        }
    }

    pub async fn send(self) -> Result<Option<Entity>, DatabaseError> {
        let cypher = "
            MATCH (e:Entity {id: $entity_id})
            OPTIONAL MATCH (e)-[p:PROPERTIES]->(props:Properties)
            WITH e, collect(p.space_id) AS spaces, collect(apoc.map.removeKey(properties(props), 'embedding')) AS props
            RETURN {entity_id: e.id, spaces: spaces, properties: props}
        ";

        let query = neo4rs::query(cypher).param("entity_id", self.entity_id.to_string());

        let mut result = self.neo4j.execute(query).await?;

        if let Some(row) = result.next().await? {
            println!("Found entity with ID: {} row: {:?}", self.entity_id, row);
            let query_result: EntityQueryResult = row.to()?;

            let entity = Entity {
                id: query_result.entity_id,
                values: query_result
                    .spaces
                    .into_iter()
                    .zip(query_result.properties)
                    .map(|(space_id, props)| {
                        (
                            space_id,
                            props
                                .into_iter()
                                .map(|(key, value)| Value {
                                    property: key,
                                    value,
                                    options: None,
                                })
                                .collect(),
                        )
                    })
                    .collect(),
            };

            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }
}

pub fn find_one(neo4j: &neo4rs::Graph, entity_id: Uuid) -> FindOneQuery {
    FindOneQuery::new(neo4j, entity_id)
}
