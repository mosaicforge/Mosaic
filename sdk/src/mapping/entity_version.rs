use futures::TryStreamExt;
use serde::Deserialize;

use crate::{error::DatabaseError, indexer_ids};

use super::{query_utils::QueryPart, PropFilter, Query};

#[derive(Debug, Deserialize, PartialEq)]
pub struct EntityVersion {
    pub entity_id: String,
    pub id: String,
    pub index: String,
}

pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    entity_id: String,
    space_id: Option<PropFilter<String>>,
}

pub fn find_many(neo4j: neo4rs::Graph, entity_id: impl Into<String>) -> FindManyQuery {
    FindManyQuery::new(neo4j, entity_id.into())
}

impl FindManyQuery {
    pub fn new(neo4j: neo4rs::Graph, entity_id: String) -> Self {
        Self {
            neo4j,
            entity_id,
            space_id: None,
        }
    }

    pub fn space_id(mut self, space_id: PropFilter<String>) -> Self {
        self.space_id = Some(space_id);
        self
    }
}

impl Query<Vec<EntityVersion>> for FindManyQuery {
    async fn send(self) -> Result<Vec<EntityVersion>, DatabaseError> {
        // const QUERY: &str = r#"
        //     MATCH (:Entity {id: $id}) -[r:ATTRIBUTE]-> (:Attribute)
        //     WHERE r.space_id = $space_id
        //     WITH COLLECT(DISTINCT r.min_version) AS versions
        //     UNWIND versions AS version
        //     MATCH (e:Entity) -[:ATTRIBUTE]-> ({id: $EDIT_INDEX_ATTR, value: version})
        //     RETURN {entity_id: $id, id: e.id, index: version}
        // "#;
        let mut query = QueryPart::default()
            .match_clause("(:Entity {id: $id}) -[r:ATTRIBUTE]-> (:Attribute)")
            .with_clause("COLLECT(DISTINCT r.min_version) AS versions", QueryPart::default()
                .unwind_clause("versions AS version")
                .match_clause("(e:Entity) -[:ATTRIBUTE]-> ({id: $EDIT_INDEX_ATTR, value: version})")
                .return_clause("{entity_id: $id, id: e.id, index: version}"))
                .params("id", self.entity_id.clone())
                .params("EDIT_INDEX_ATTR", indexer_ids::EDIT_INDEX_ATTRIBUTE);

        if let Some(space_id) = self.space_id {
            query.merge_mut(space_id.into_query_part("r", "space_id"))
        }

        let query= query.build();

        self.neo4j
            .execute(query)
            .await?
            .into_stream_as::<EntityVersion>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move { Ok(row) })
            .try_collect::<Vec<_>>()
            .await
    }
}
