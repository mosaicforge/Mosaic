use serde::{Deserialize, Serialize};

use crate::{error::DatabaseError, ids, indexer_ids, mapping::Relation, system_ids};

/// Space editor relation.
/// Account > EDITOR > Space
#[derive(Clone, Deserialize, Serialize)]
pub struct SpaceEditor;

impl SpaceEditor {
    pub fn new(editor_id: &str, space_id: &str) -> Relation<Self> {
        Relation::new(
            &ids::create_id_from_unique_string(&format!("EDITOR:{space_id}:{editor_id}")),
            editor_id,
            space_id,
            indexer_ids::EDITOR_RELATION,
            "0",
            Self,
        )
    }

    /// Returns a query to delete a relation between an editor and a space.
    pub async fn remove(
        neo4j: &neo4rs::Graph,
        editor_id: &str,
        space_id: &str,
    ) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
                MATCH ({{id: $from}})<-[:`{FROM_ENTITY}`]-(r)-[:`{TO_ENTITY}`]->({{id: $to}})
                MATCH (r) -[:`{RELATION_TYPE}`]-> ({{id: $relation_type}})
                DETACH DELETE r
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
        );

        let query = neo4rs::query(QUERY)
            .param("from", editor_id)
            .param("to", space_id)
            .param("relation_type", indexer_ids::EDITOR_RELATION);

        Ok(neo4j.run(query).await?)
    }
}
