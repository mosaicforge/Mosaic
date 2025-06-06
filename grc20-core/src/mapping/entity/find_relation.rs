use std::collections::HashMap;

use neo4rs::{BoltType, Relation};
use serde::{Deserialize, Serialize};

use crate::{
    entity::EntityFilter,
    error::DatabaseError,
    mapping::{
        order_by::FieldOrderBy,
        query_utils::{
            query_builder::{MatchQuery, QueryBuilder, Subquery},
            VersionFilter,
        },
        AttributeFilter, FromAttributes, PropFilter, Query,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectedRelationship {
    pub identity: i64,
    pub start: i64,
    pub end: i64,
    pub rel_type: String,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Properties {
    min_version: String,
    max_version: Option<String>,
    #[serde(rename = "7pXCVQDV9C7ozrXkpVg8RJ")]
    _updated_at: String,
    index: String,
    #[serde(rename = "82nP7aFmHJLbaPFszj2nbx")]
    _created: String,
    #[serde(rename = "5Ms1pYq8v8G1RXC3wWb9ix")]
    _updated: String,
    relation_type: String,
    id: String,
    #[serde(rename = "59HTYnd2e4gBx2aA98JfNx")]
    _created_at_block: String,
    space_id: String,
}

pub struct FindRelationQuery<T> {
    neo4j: neo4rs::Graph,
    id1: String,
    id2: String,
    filter: EntityFilter,
    order_by: Option<FieldOrderBy>,
    limit: usize,
    skip: Option<usize>,
    space_id: Option<PropFilter<String>>,
    version: VersionFilter,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> FindRelationQuery<T> {
    pub(super) fn new(neo4j: &neo4rs::Graph, id1: String, id2: String) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id1,
            id2,
            filter: EntityFilter::default(),
            order_by: None,
            limit: 100,
            skip: None,
            space_id: None,
            version: VersionFilter::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn id(mut self, id: PropFilter<String>) -> Self {
        self.filter.id = Some(id);
        self
    }

    pub fn attribute(mut self, attribute: AttributeFilter) -> Self {
        self.filter.attributes.push(attribute);
        self
    }

    pub fn attribute_mut(&mut self, attribute: AttributeFilter) {
        self.filter.attributes.push(attribute);
    }

    pub fn attributes(mut self, attributes: impl IntoIterator<Item = AttributeFilter>) -> Self {
        self.filter.attributes.extend(attributes);
        self
    }

    pub fn attributes_mut(&mut self, attributes: impl IntoIterator<Item = AttributeFilter>) {
        self.filter.attributes.extend(attributes);
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    /// Overwrite the current filter with a new one
    pub fn with_filter(mut self, filter: EntityFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn order_by(mut self, order_by: FieldOrderBy) -> Self {
        self.order_by = Some(order_by);
        self
    }

    pub fn order_by_mut(&mut self, order_by: FieldOrderBy) {
        self.order_by = Some(order_by);
    }

    pub fn space_id(mut self, space_id: impl Into<PropFilter<String>>) -> Self {
        self.space_id = Some(space_id.into());
        self
    }

    pub fn version(mut self, space_version: String) -> Self {
        self.version.version_mut(space_version);
        self
    }

    pub fn version_opt(mut self, space_version: Option<String>) -> Self {
        self.version.version_opt(space_version);
        self
    }

    fn subquery(&self) -> QueryBuilder {
        QueryBuilder::default()
            .subquery(MatchQuery::new(
                "e = (e1:Entity {id: $id1}) -[:RELATION*1..3]-(e2:Entity {id: $id2})",
            ))
            .limit(self.limit)
            .params("id1", self.id1.clone())
            .params("id2", self.id2.clone())
    }
}

impl<T: FromAttributes> Query<Vec<Vec<ConnectedRelationship>>> for FindRelationQuery<T> {
    async fn send(self) -> Result<Vec<Vec<ConnectedRelationship>>, DatabaseError> {
        let query = self.subquery().r#return("relationships(e)");

        if cfg!(debug_assertions) || cfg!(test) {
            println!(
                "entity::FindRelationQuery::<T>:\n{}\nparams:{:?}",
                query.compile(),
                [self.id1, self.id2]
            );
        };

        let mut result = self.neo4j.execute(query.build()).await?;
        let mut all_relationship_paths = Vec::new();

        // Process each row
        while let Some(row) = result.next().await? {
            // Get the relationships collection from the row - note the key name
            let relationships: Vec<Relation> = row.get("relationships(e)")?;

            let mut path_relationships = Vec::new();

            // Convert each Neo4j Relation to our custom struct
            for rel in relationships {
                let relationship = ConnectedRelationship {
                    identity: rel.id(),
                    start: rel.start_node_id(),
                    end: rel.end_node_id(),
                    rel_type: rel.typ().to_string(),
                    properties: convert_properties(&rel),
                };
                path_relationships.push(relationship);
            }

            all_relationship_paths.push(path_relationships);
        }

        Ok(all_relationship_paths)
        /*Row {
        attributes: BoltMap {
        value: {BoltString { value: "relationships(e)" }: List(BoltList { value: [
            Relation(BoltRelation { id: BoltInteger { value: 74 }, start_node_id: BoltInteger { value: 4 }, end_node_id: BoltInteger { value: 1836 }, typ: BoltString { value: "RELATION" }, properties: BoltMap {
                value: {
                    BoltString { value: "index" }: String(BoltString { value: "0" }),
                    BoltString { value: "relation_type" }: String(BoltString { value: "Jfmby78N4BCseZinBmdVov" }),
                    BoltString { value: "id" }: String(BoltString { value: "AqysdC1YZt2iBkCL2qgyg5" }),
                    BoltString { value: "7pXCVQDV9C7ozrXkpVg8RJ" }: String(BoltString { value: "0" }),
                    BoltString { value: "82nP7aFmHJLbaPFszj2nbx" }: DateTime(BoltDateTime { seconds: BoltInteger { value: 0 }, nanoseconds: BoltInteger { value: 0 }, tz_offset_seconds: BoltInteger { value: 0 } }),
                    BoltString { value: "min_version" }: String(BoltString { value: "0" }), BoltString { value: "5Ms1pYq8v8G1RXC3wWb9ix" }: DateTime(BoltDateTime { seconds: BoltInteger { value: 0 }, nanoseconds: BoltInteger { value: 0 }, tz_offset_seconds: BoltInteger { value: 0 } }),
                    BoltString { value: "59HTYnd2e4gBx2aA98JfNx" }: String(BoltString { value: "0" }), BoltString { value: "space_id" }: String(BoltString { value: "25omwWh6HYgeRQKCaSpVpa" })
                } } }),
            Relation(BoltRelation { id: BoltInteger { value: 77 }, start_node_id: BoltInteger { value: 7 }, end_node_id: BoltInteger { value: 1836 }, typ: BoltString { value: "RELATION" }, properties: BoltMap { value: {BoltString { value: "id" }: String(BoltString { value: "QmqmtmDj4jAY5v3dArk2av" }), BoltString { value: "82nP7aFmHJLbaPFszj2nbx" }: DateTime(BoltDateTime { seconds: BoltInteger { value: 0 }, nanoseconds: BoltInteger { value: 0 }, tz_offset_seconds: BoltInteger { value: 0 } }), BoltString { value: "7pXCVQDV9C7ozrXkpVg8RJ" }: String(BoltString { value: "0" }), BoltString { value: "59HTYnd2e4gBx2aA98JfNx" }: String(BoltString { value: "0" }), BoltString { value: "5Ms1pYq8v8G1RXC3wWb9ix" }: DateTime(BoltDateTime { seconds: BoltInteger { value: 0 }, nanoseconds: BoltInteger { value: 0 }, tz_offset_seconds: BoltInteger { value: 0 } }), BoltString { value: "min_version" }: String(BoltString { value: "0" }), BoltString { value: "index" }: String(BoltString { value: "0" }), BoltString { value: "relation_type" }: String(BoltString { value: "Jfmby78N4BCseZinBmdVov" }), BoltString { value: "space_id" }: String(BoltString { value: "25omwWh6HYgeRQKCaSpVpa" })} } })
        ] })} } }*/
    }
}

fn convert_properties(rel: &Relation) -> HashMap<String, serde_json::Value> {
    let mut properties = HashMap::new();

    for key in rel.keys() {
        if let Ok(value) = rel.get::<BoltType>(key) {
            match value {
                BoltType::String(s) => {
                    properties.insert(key.to_string(), serde_json::Value::String(s.value));
                }
                BoltType::Integer(i) => {
                    properties.insert(
                        key.to_string(),
                        serde_json::Value::Number(serde_json::Number::from(i.value)),
                    );
                }
                BoltType::Float(f) => {
                    if let Some(num) = serde_json::Number::from_f64(f.value) {
                        properties.insert(key.to_string(), serde_json::Value::Number(num));
                    }
                }
                BoltType::Boolean(b) => {
                    properties.insert(key.to_string(), serde_json::Value::Bool(b.value));
                }
                BoltType::DateTime(_) => {
                    properties.insert(
                        key.to_string(),
                        serde_json::Value::String("1970-01-01T00:00:00Z".to_string()),
                    );
                }
                BoltType::List(_list) => {
                    properties.insert(key.to_string(), serde_json::Value::Array(vec![]));
                }
                _ => {
                    properties.insert(key.to_string(), serde_json::Value::Null);
                }
            }
        }
    }

    properties
}
