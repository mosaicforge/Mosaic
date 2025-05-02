use crate::{
    block::BlockMetadata,
    error::DatabaseError,
    indexer_ids,
    mapping::{EntityNodeRef, Query},
};

use super::RelationEdge;

pub struct InsertManyQuery<T> {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: String,
    relations: Vec<T>,
}

impl<T> InsertManyQuery<T> {
    pub(super) fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: String,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            relations: vec![],
        }
    }

    pub fn relation(mut self, relation: T) -> Self {
        self.relations.push(relation);
        self
    }

    pub fn relation_mut(&mut self, relation: T) {
        self.relations.push(relation);
    }

    pub fn relations(mut self, relations: impl IntoIterator<Item = T>) -> Self {
        self.relations.extend(relations);
        self
    }

    pub fn relations_mut(&mut self, relations: impl IntoIterator<Item = T>) {
        self.relations.extend(relations);
    }
}

impl Query<()> for InsertManyQuery<RelationEdge<EntityNodeRef>> {
    async fn send(self) -> Result<(), DatabaseError> {
        if self.relations.is_empty() {
            return Ok(());
        }

        const QUERY: &str = const_format::formatcp!(
            r#"
            UNWIND $relations as relation
            MATCH (from:Entity {{id: relation.from}})
            MATCH (to:Entity {{id: relation.to}})
            CREATE (from) -[r:RELATION]-> (to)
            SET r += {{
                id: relation.id,
                space_id: $space_id,
                index: relation.index,
                min_version: $space_version,
                relation_type: relation.relation_type,
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number,
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            "#,
            CREATED_AT = indexer_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = indexer_ids::CREATED_AT_BLOCK,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        if cfg!(debug_assertions) || cfg!(test) {
            tracing::info!("relation_node::InsertManyQuery:\n{}", QUERY);
            tracing::info!("relations:\n{:#?}", self.relations);
        };

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("relations", self.relations)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use futures::TryStreamExt;

    use crate::{
        block::BlockMetadata,
        mapping::{
            prop_filter,
            relation::{find_many, insert_many, RelationEdge},
            triple, EntityFilter, EntityNodeRef, Query, QueryStream, Triple,
        },
        relation::utils::RelationFilter,
    };

    #[tokio::test]
    async fn test_insert_many_find_many() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        triple::insert_many(&neo4j, &BlockMetadata::default(), "space_id", "0")
            .triples(vec![
                Triple::new("alice", "name", "Alice"),
                Triple::new("bob", "name", "Bob"),
                Triple::new("knows", "name", "knows"),
                Triple::new("charlie", "name", "Charlie"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation_nodes = vec![
            RelationEdge::new("abc", "alice", "bob", "knows", "0"),
            RelationEdge::new("dev", "alice", "charlie", "knows", "1"),
        ];

        insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .relations(relation_nodes.clone())
            .send()
            .await
            .expect("Failed to insert relations");

        let found_relations = find_many::<RelationEdge<EntityNodeRef>>(&neo4j)
            .filter(
                RelationFilter::default()
                    .relation_type(EntityFilter::default().id(prop_filter::value("knows")))
                    .from_(EntityFilter::default().id(prop_filter::value("alice"))),
            )
            .send()
            .await
            .expect("Failed to find relations")
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to collect triples");

        assert_eq!(found_relations, relation_nodes);
    }
}
