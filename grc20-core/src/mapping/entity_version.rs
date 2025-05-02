use futures::TryStreamExt;
use serde::Deserialize;

use crate::{error::DatabaseError, indexer_ids};

use super::{
    query_utils::query_builder::{MatchQuery, QueryBuilder, Subquery},
    PropFilter, Query,
};

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
        let query = QueryBuilder::default()
            .subquery(
                MatchQuery::new("(:Entity {id: $id}) -[r:ATTRIBUTE]-> (:Attribute)").where_opt(
                    self.space_id
                        .as_ref()
                        .map(|s| s.subquery("r", "space_id", None)),
                ),
            )
            .with(
                vec!["COLLECT(DISTINCT r.min_version) AS versions".to_string()],
                QueryBuilder::default()
                    .subquery("versions AS version")
                    .subquery(MatchQuery::new(
                        "(e:Entity) -[:ATTRIBUTE]-> ({id: $EDIT_INDEX_ATTR, value: version})",
                    ))
                    .subquery("RETURN {entity_id: $id, id: e.id, index: version}"),
            )
            .params("id", self.entity_id.clone())
            .params("EDIT_INDEX_ATTR", indexer_ids::EDIT_INDEX_ATTRIBUTE);

        if cfg!(debug_assertions) || cfg!(test) {
            println!(
                "entity_version::FindManyQuery::<EntityVersion>:\n{}\nparams:{:?}",
                query.compile(),
                query.params
            );
        }

        self.neo4j
            .execute(query.build())
            .await?
            .into_stream_as::<EntityVersion>()
            .map_err(DatabaseError::from)
            .and_then(|row| async move { Ok(row) })
            .try_collect::<Vec<_>>()
            .await
    }
}
