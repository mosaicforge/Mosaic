use crate::{
    block::BlockMetadata,
    error::DatabaseError,
    indexer_ids,
    mapping::{EntityNodeRef, IntoAttributes, Query},
};

use super::{Relation, RelationEdge};

pub struct InsertOneQuery<T> {
    neo4j: neo4rs::Graph,
    block: BlockMetadata,
    space_id: String,
    space_version: String,
    relation: T,
}

impl<T> InsertOneQuery<T> {
    pub(super) fn new(
        neo4j: &neo4rs::Graph,
        block: &BlockMetadata,
        space_id: String,
        space_version: String,
        relation: T,
    ) -> Self {
        Self {
            neo4j: neo4j.clone(),
            block: block.clone(),
            space_id,
            space_version,
            relation,
        }
    }
}

impl Query<()> for InsertOneQuery<RelationEdge<EntityNodeRef>> {
    async fn send(self) -> Result<(), DatabaseError> {
        // TODO: Add old relation deletion
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (from:Entity {{id: $relation.from}})
            MATCH (to:Entity {{id: $relation.to}})
            CREATE (from) -[r:RELATION]-> (to)
            SET r += {{
                id: $relation.id,
                space_id: $space_id,
                index: $relation.index,
                min_version: $space_version,
                relation_type: $relation.relation_type,
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

        let query = neo4rs::query(QUERY)
            .param("space_id", self.space_id)
            .param("space_version", self.space_version)
            .param("relation", self.relation)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

impl<T: IntoAttributes> Query<()> for InsertOneQuery<Relation<T, EntityNodeRef>> {
    async fn send(self) -> Result<(), DatabaseError> {
        const QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (from:Entity {{id: $relation.from}})
            MATCH (to:Entity {{id: $relation.to}})
            MERGE (from) -[r:RELATION]-> (to)
            ON CREATE SET r += {{
                id: $relation.id,
                space_id: $space_id,
                index: $relation.index,
                min_version: $space_version,
                relation_type: $relation.relation_type,
                `{CREATED_AT}`: datetime($block_timestamp),
                `{CREATED_AT_BLOCK}`: $block_number
            }}
            SET r += {{
                `{UPDATED_AT}`: datetime($block_timestamp),
                `{UPDATED_AT_BLOCK}`: $block_number
            }}
            MERGE (e:Entity {{id: $relation.id}})
            WITH e
            UNWIND $attributes AS attribute
            CALL (e, attribute) {{
                MATCH (e) -[r:ATTRIBUTE {{space_id: $space_id}}]-> (:Attribute {{id: attribute.id}})
                WHERE r.max_version IS null AND r.min_version <> $space_version
                SET r.max_version = $space_version
            }}
            CALL (e, attribute) {{
                MERGE (e) -[:ATTRIBUTE {{space_id: $space_id, min_version: $space_version}}]-> (m:Attribute {{id: attribute.id}})
                SET m += attribute
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
            .param("relation", self.relation.relation)
            .param("attributes", self.relation.attributes.into_attributes()?)
            .param("block_number", self.block.block_number.to_string())
            .param("block_timestamp", self.block.timestamp.to_rfc3339());

        self.neo4j.run(query).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        block::BlockMetadata,
        mapping::{
            self,
            relation::{find_one, RelationEdge},
            triple, EntityNode, EntityNodeRef, Query, Relation, Triple,
        },
        system_ids,
    };

    #[derive(Clone, Debug, PartialEq)]
    struct Foo {
        name: String,
        bar: u64,
    }

    impl mapping::IntoAttributes for Foo {
        fn into_attributes(self) -> Result<mapping::Attributes, mapping::TriplesConversionError> {
            Ok(mapping::Attributes::default()
                .attribute(("name", self.name))
                .attribute(("bar", self.bar)))
        }
    }

    impl mapping::FromAttributes for Foo {
        fn from_attributes(
            mut attributes: mapping::Attributes,
        ) -> Result<Self, mapping::TriplesConversionError> {
            Ok(Self {
                name: attributes.pop("name")?,
                bar: attributes.pop("bar")?,
            })
        }
    }

    #[tokio::test]
    async fn test_insert_find_one() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        triple::insert_many(&neo4j, &BlockMetadata::default(), "space_id", "0")
            .triples(vec![
                Triple::new("alice", "name", "Alice"),
                Triple::new("bob", "name", "Bob"),
                Triple::new("knows", "name", "knows"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation_node = RelationEdge::new("abc", "alice", "bob", "knows", "0");

        relation_node
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let found_relation: RelationEdge<EntityNodeRef> =
            find_one::<RelationEdge<EntityNodeRef>>(&neo4j, "abc", "ROOT", None)
                .send()
                .await
                .expect("Failed to find relation")
                .expect("Relation not found");

        assert_eq!(found_relation, relation_node);
    }

    #[tokio::test]
    async fn test_insert_find_one_node() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        triple::insert_many(&neo4j, &BlockMetadata::default(), "space_id", "0")
            .triples(vec![
                Triple::new("alice", "name", "Alice"),
                Triple::new("bob", "name", "Bob"),
                Triple::new("knows", "name", "knows"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation_node = RelationEdge::new("abc", "alice", "bob", "knows", "0");

        relation_node
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let found_relation = find_one::<RelationEdge<EntityNode>>(&neo4j, "abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find relation")
            .expect("Relation not found");

        assert_eq!(
            found_relation,
            RelationEdge {
                id: "abc".to_string(),
                from: EntityNode {
                    id: "alice".to_string(),
                    system_properties: BlockMetadata::default().into(),
                },
                to: EntityNode {
                    id: "bob".to_string(),
                    system_properties: BlockMetadata::default().into(),
                },
                relation_type: "knows".to_string(),
                index: "0".to_string(),
                system_properties: BlockMetadata::default().into(),
            },
        );
    }

    #[tokio::test]
    async fn test_insert_find_one_relation() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        let foo = Foo {
            name: "Alice".into(),
            bar: 42,
        };

        triple::insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .triples(vec![
                Triple::new("from_id", "name", "FooFrom"),
                Triple::new("to_id", "name", "FooTo"),
                Triple::new("relation_type", "name", "FooRelation"),
                Triple::new(system_ids::TYPES_ATTRIBUTE, "name", "Types"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation = Relation::new("rel_abc", "from_id", "to_id", "relation_type", 0u64, foo);

        relation
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let found_relation =
            find_one::<Relation<Foo, EntityNodeRef>>(&neo4j, "rel_abc", "ROOT", None)
                .send()
                .await
                .expect("Failed to find relation")
                .expect("Relation not found");

        assert_eq!(found_relation, relation);
    }

    #[tokio::test]
    async fn test_insert_find_one_relation_node() {
        // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
        let (_container, neo4j) = crate::test_utils::setup_neo4j().await;

        let foo = Foo {
            name: "Alice".into(),
            bar: 42,
        };

        triple::insert_many(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .triples(vec![
                Triple::new("from_id", "name", "FooFrom"),
                Triple::new("to_id", "name", "FooTo"),
                Triple::new("relation_type", "name", "FooRelation"),
                Triple::new(system_ids::TYPES_ATTRIBUTE, "name", "Types"),
            ])
            .send()
            .await
            .expect("Failed to insert triples");

        let relation = Relation::new(
            "rel_abc",
            "from_id",
            "to_id",
            "relation_type",
            0u64,
            foo.clone(),
        );

        relation
            .clone()
            .insert(&neo4j, &BlockMetadata::default(), "ROOT", "0")
            .send()
            .await
            .expect("Failed to insert relation");

        let found_relation = find_one::<Relation<Foo, EntityNode>>(&neo4j, "rel_abc", "ROOT", None)
            .send()
            .await
            .expect("Failed to find relation")
            .expect("Relation not found");

        assert_eq!(
            found_relation,
            Relation {
                relation: RelationEdge {
                    id: "rel_abc".to_string(),
                    from: EntityNode {
                        id: "from_id".to_string(),
                        system_properties: BlockMetadata::default().into(),
                    },
                    to: EntityNode {
                        id: "to_id".to_string(),
                        system_properties: BlockMetadata::default().into(),
                    },
                    relation_type: "relation_type".to_string(),
                    index: "0".to_string(),
                    system_properties: BlockMetadata::default().into(),
                },
                attributes: foo,
            },
        );
    }
}
