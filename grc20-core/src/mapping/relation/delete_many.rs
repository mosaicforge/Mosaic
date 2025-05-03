use crate::{block::BlockMetadata, error::DatabaseError, indexer_ids, mapping::Query};

pub struct DeleteManyQuery {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: String,
    relations: Vec<String>,
}

impl DeleteManyQuery {
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

    pub fn relation(mut self, relation_id: impl Into<String>) -> Self {
        self.relations.push(relation_id.into());
        self
    }

    pub fn relation_mut(&mut self, relation_id: impl Into<String>) {
        self.relations.push(relation_id.into());
    }

    pub fn relations(mut self, relation_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.relations
            .extend(relation_ids.into_iter().map(Into::into));
        self
    }

    pub fn relations_mut(&mut self, relation_ids: impl IntoIterator<Item = impl Into<String>>) {
        self.relations
            .extend(relation_ids.into_iter().map(Into::into));
    }
}

impl Query<()> for DeleteManyQuery {
    async fn send(self) -> Result<(), DatabaseError> {
        // TODO: Add relation entity deletion
        const QUERY: &str = const_format::formatcp!(
            r#"
                UNWIND $relations as relation_id
                MATCH () -[r:RELATION {{id: relation_id}}]-> ()
                WHERE r.space_id = $space_id
                AND r.max_version IS NULL
                SET r.max_version = $space_version
                SET r += {{
                    `{UPDATED_AT}`: datetime($block_timestamp),
                    `{UPDATED_AT_BLOCK}`: $block_number
                }}
            "#,
            UPDATED_AT = indexer_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = indexer_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("relations", self.relations)
            .param("block_timestamp", self.block.timestamp.to_rfc3339())
            .param("block_number", self.block.block_number.to_string());

        Ok(self.neo4j.run(query).await?)
    }
}
