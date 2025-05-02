use crate::{
    error::DatabaseError,
    mapping::{
        query_utils::{
            query_builder::{MatchQuery, QueryBuilder, Subquery},
            VersionFilter,
        },
        AttributeNode, EntityNode, EntityNodeRef, FromAttributes, Query,
    },
};

use super::{FindOneToQuery, Relation, RelationEdge};

pub struct FindOneQuery<T> {
    neo4j: neo4rs::Graph,
    id: String,
    space_id: String,
    version: VersionFilter,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> FindOneQuery<T> {
    pub(super) fn new(
        neo4j: &neo4rs::Graph,
        id: String,
        space_id: String,
        space_version: Option<String>,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            id,
            space_id,
            version: VersionFilter::new(space_version),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn select_to<U>(self) -> FindOneToQuery<U> {
        FindOneToQuery {
            neo4j: self.neo4j,
            id: self.id,
            space_id: self.space_id,
            version: self.version,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Query<Option<RelationEdge<EntityNodeRef>>> for FindOneQuery<RelationEdge<EntityNodeRef>> {
    async fn send(self) -> Result<Option<RelationEdge<EntityNodeRef>>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query = QueryBuilder::default()
            .subquery(
                MatchQuery::new(
                    "(from:Entity) -[r:RELATION {id: $id, space_id: $space_id}]-> (to:Entity)",
                )
                .r#where(self.version.subquery("r"))
                .params("id", self.id)
                .params("space_id", self.space_id),
            )
            .subquery("ORDER BY r.index")
            .r#return("r{.*, from: from.id, to: to.id} as r");

        neo4j
            .execute(query.build())
            .await?
            .next()
            .await?
            .map(|row| Result::<_, DatabaseError>::Ok(row.to::<RelationEdge<EntityNodeRef>>()?))
            .transpose()
    }
}

impl Query<Option<RelationEdge<EntityNode>>> for FindOneQuery<RelationEdge<EntityNode>> {
    async fn send(self) -> Result<Option<RelationEdge<EntityNode>>, DatabaseError> {
        let neo4j = self.neo4j.clone();
        let query = QueryBuilder::default()
            .subquery(
                MatchQuery::new(
                    "(from:Entity) -[r:RELATION {id: $id, space_id: $space_id}]-> (to:Entity)",
                )
                .r#where(self.version.subquery("r"))
                .params("id", self.id)
                .params("space_id", self.space_id),
            )
            .subquery("ORDER BY r.index")
            .r#return("r{.*, from: from, to: to} as r");

        neo4j
            .execute(query.build())
            .await?
            .next()
            .await?
            .map(|row| Result::<_, DatabaseError>::Ok(row.to::<RelationEdge<EntityNode>>()?))
            .transpose()
    }
}

impl<T: FromAttributes> Query<Option<Relation<T, EntityNodeRef>>>
    for FindOneQuery<Relation<T, EntityNodeRef>>
{
    async fn send(self) -> Result<Option<Relation<T, EntityNodeRef>>, DatabaseError> {
        let query = QueryBuilder::default()
            .subquery(
                MatchQuery::new(
                    "(from:Entity) -[r:RELATION {id: $id, space_id: $space_id}]-> (to:Entity)",
                )
                .r#where(self.version.subquery("r"))
                .params("id", self.id)
                .params("space_id", self.space_id),
            )
            .subquery("ORDER BY r.index")
            .with(
                vec!["r".to_string(), "from".to_string(), "to".to_string()],
                QueryBuilder::default()
                    .subquery(MatchQuery::new("(r_e:Entity {id: r.id})"))
                    .subquery(
                        MatchQuery::new_optional(
                            "(r_e) -[r_attr:ATTRIBUTE {space_id: $space_id}]-> (n:Attribute)",
                        )
                        .r#where(self.version.subquery("r_attr")),
                    )
                    .with(
                        vec![
                            "r".to_string(),
                            "r_e".to_string(),
                            "from".to_string(),
                            "to".to_string(),
                            "COLLECT(DISTINCT n{.*}) AS attrs".to_string(),
                        ],
                        "RETURN r{.*, from: from.id, to: to.id, attributes: attrs} as r",
                    ),
            );

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            edge: RelationEdge<EntityNodeRef>,
            attributes: Vec<AttributeNode>,
        }

        self.neo4j
            .execute(query.build())
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                Result::<_, DatabaseError>::Ok(Relation {
                    relation: row.edge,
                    attributes: T::from_attributes(row.attributes.into())?,
                })
            })
            .transpose()
    }
}

impl<T: FromAttributes> Query<Option<Relation<T, EntityNode>>>
    for FindOneQuery<Relation<T, EntityNode>>
{
    async fn send(self) -> Result<Option<Relation<T, EntityNode>>, DatabaseError> {
        let query = QueryBuilder::default()
            .subquery(
                MatchQuery::new(
                    "(from:Entity) -[r:RELATION {id: $id, space_id: $space_id}]-> (to:Entity)",
                )
                .r#where(self.version.subquery("r"))
                .params("id", self.id)
                .params("space_id", self.space_id),
            )
            .subquery("ORDER BY r.index")
            .with(
                vec!["r".to_string(), "from".to_string(), "to".to_string()],
                QueryBuilder::default()
                    .subquery(MatchQuery::new("(r_e:Entity {id: r.id})"))
                    .subquery(
                        MatchQuery::new_optional(
                            "(r_e) -[r_attr:ATTRIBUTE {space_id: $space_id}]-> (n:Attribute)",
                        )
                        .r#where(self.version.subquery("r_attr")),
                    )
                    .with(
                        vec![
                            "r".to_string(),
                            "r_e".to_string(),
                            "from".to_string(),
                            "to".to_string(),
                            "COLLECT(DISTINCT n{.*}) AS attrs".to_string(),
                        ],
                        "RETURN r{.*, from: from, to: to, attributes: attrs} as r",
                    ),
            );

        #[derive(Debug, serde::Deserialize)]
        struct RowResult {
            #[serde(flatten)]
            edge: RelationEdge<EntityNode>,
            attributes: Vec<AttributeNode>,
        }

        self.neo4j
            .execute(query.build())
            .await?
            .next()
            .await?
            .map(|row| {
                let row = row.to::<RowResult>()?;
                Result::<_, DatabaseError>::Ok(Relation {
                    relation: row.edge,
                    attributes: T::from_attributes(row.attributes.into())?,
                })
            })
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        block::BlockMetadata,
        indexer_ids,
        mapping::{relation::find_one, EntityNodeRef, Query, RelationEdge},
    };

    #[tokio::test]
    async fn test_find_one() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        let block = &BlockMetadata::default();

        neo4j.run(neo4rs::query(&format!(
            r#"
            CREATE (alice:Entity {{id: "alice"}})
            CREATE (bob:Entity {{id: "bob"}})
            CREATE (knows:Entity {{id: "knows"}})
            CREATE (alice) -[r:RELATION {{id: "abc", relation_type: "knows", space_id: "ROOT", min_version: "0", index: "0"}}]-> (bob)
            SET r += {{
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
        ))
            .param("block_timestamp", block.timestamp.to_rfc3339())
            .param("block_number", block.block_number.to_string())
        )
            .await
            .expect("Failed to insert data");

        let relation_node = RelationEdge::new("abc", "alice", "bob", "knows", "0");

        let found_relation = find_one::<RelationEdge<EntityNodeRef>>(&neo4j, "abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find relation")
            .expect("Relation not found");

        assert_eq!(found_relation, relation_node);
    }
}
