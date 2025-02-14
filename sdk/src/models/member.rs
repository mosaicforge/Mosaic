use serde::{Deserialize, Serialize};

use crate::{error::DatabaseError, ids, indexer_ids, mapping::Relation, system_ids};

/// Space editor relation.
#[derive(Clone, Deserialize, Serialize)]
pub struct SpaceMember;

impl SpaceMember {
    pub fn new(member_id: &str, space_id: &str) -> Relation<Self> {
        Relation::new(
            &ids::create_id_from_unique_string(&format!("MEMBER:{space_id}:{member_id}")),
            member_id,
            space_id,
            indexer_ids::MEMBER_RELATION,
            "0",
            Self,
        )
    }

    /// Returns a query to delete a relation between an member and a space.
    pub async fn remove(
        neo4j: &neo4rs::Graph,
        member_id: &str,
        space_id: &str,
    ) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
                MATCH ({{id: $from}})<-[:`{FROM_ENTITY}`]-(r)-[:`{TO_ENTITY}`]->({{id: $to}})
                MATCH (r) -[:`{RELATION_TYPE}`]->({{id: $relation_type}})
                DETACH DELETE r
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
        );

        let query = neo4rs::query(QUERY)
            .param("from", member_id)
            .param("to", space_id)
            .param("relation_type", indexer_ids::MEMBER_RELATION);

        Ok(neo4j.run(query).await?)
    }
}
