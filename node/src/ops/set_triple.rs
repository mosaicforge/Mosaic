use crate::{kg::entity::EntityNode, system_ids};

use crate::ops::{KgOp, Value};

pub struct SetTriple {
    pub entity_id: String,
    pub attribute_id: String,
    pub value: Value,
}

impl KgOp for SetTriple {
    async fn apply_op(&self, kg: &crate::kg::client::Client, space_id: &str) -> anyhow::Result<()> {
        let entity_name = kg
            .get_name(&self.entity_id)
            .await?
            .unwrap_or(self.entity_id.to_string());

        let attribute_name = kg
            .get_name(&self.attribute_id)
            .await?
            .unwrap_or(self.attribute_id.to_string());

        tracing::info!(
            "SetTriple: {}, {}, {}",
            if entity_name == self.entity_id {
                self.entity_id.to_string()
            } else {
                format!("{} ({})", entity_name, self.entity_id)
            },
            if attribute_name == self.attribute_id {
                self.attribute_id.to_string()
            } else {
                format!("{} ({})", attribute_name, self.attribute_id)
            },
            self.value,
        );

        match (self.attribute_id.as_str(), &self.value) {
            (system_ids::TYPES, Value::Entity(value)) => {
                if let Some(_) = kg
                    .find_relation_by_id::<EntityNode>(&self.entity_id)
                    .await?
                {
                    // let entity = Entity::from_entity(kg.clone(), relation);
                    // kg.neo4j.run(
                    //     neo4rs::query(&format!(
                    //         r#"
                    //         MATCH (n) -[{{id: $relation_id}}]-> (m)
                    //         CREATE (n) -[:{relation_label} {{id: $relation_id, relation_type_id: $relation_type_id}}]-> (m)
                    //         "#,
                    //         relation_label = RelationLabel::new(value),
                    //     ))
                    //     .param("relation_id", self.entity_id.clone())
                    //     .param("relation_type_id", system_ids::TYPES),
                    // ).await?;
                    tracing::warn!(
                        "Unhandled case: Setting type on existing relation {entity_name}"
                    );
                } else {
                    kg.neo4j
                        .run(
                            neo4rs::query(&format!(
                                r#"
                                MERGE (t {{ id: $value, space_id: $space_id }})
                                MERGE (n {{ id: $id, space_id: $space_id }}) 
                                ON CREATE 
                                    SET n :`{value}`
                                ON MATCH 
                                    SET n :`{value}`
                                "#,
                                // MERGE (n) -[:TYPE {{id: $attribute_id}}]-> (t)
                                // "#,
                            ))
                            .param("id", self.entity_id.clone())
                            .param("value", self.value.clone())
                            .param("space_id", space_id),
                        )
                        .await?;
                }
            }
            // (system_ids::NAME, Value::Text(value)) => {
            //     if let Some(_) = kg.find_relation_by_id::<EntityNode>(&self.entity_id).await? {
            //         tracing::warn!("Unhandled case: Setting name on relation {entity_name}");
            //     } else {
            //         kg.set_name(&self.entity_id, &value).await?;
            //     }
            // }
            (attribute_id, Value::Entity(value)) => {
                if ![
                    system_ids::RELATION_FROM_ATTRIBUTE,
                    system_ids::RELATION_TO_ATTRIBUTE,
                    system_ids::RELATION_INDEX,
                    system_ids::RELATION_TYPE_ATTRIBUTE,
                ]
                .contains(&attribute_id)
                {
                    panic!("Unhandled case: Setting entity value on attribute {attribute_name}({attribute_id}) of entity {entity_name}({})", self.entity_id);
                }

                if let Some(_) = kg
                    .find_relation_by_id::<EntityNode>(&self.entity_id)
                    .await?
                {
                    tracing::warn!("Unhandled case: Relation {attribute_name} defined on relation {entity_name}");
                } else {
                    kg.neo4j
                        .run(
                            neo4rs::query(&format!(
                                r#"
                                MERGE (n {{ id: $id }})
                                MERGE (m {{ id: $value }})
                                MERGE (n) -[:`{attribute_id}` {{space_id: $space_id}}]-> (m)
                                "#,
                            ))
                            .param("id", self.entity_id.clone())
                            .param("value", value.clone())
                            .param("space_id", space_id),
                        )
                        .await?;
                }
            }
            (attribute_id, value) => {
                if let Some(_) = kg
                    .find_relation_by_id::<EntityNode>(&self.entity_id)
                    .await?
                {
                    kg.neo4j
                        .run(
                            neo4rs::query(&format!(
                                r#"
                            MATCH () -[r {{id: $relation_id, space_id: $space_id}}]-> ()
                            SET r.`{attribute_id}` = $value
                            "#,
                            ))
                            .param("relation_id", self.entity_id.clone())
                            .param("value", value.clone())
                            .param("space_id", space_id),
                        )
                        .await?;
                } else {
                    kg.neo4j
                        .run(
                            neo4rs::query(&format!(
                                r#"
                                MERGE (n {{ id: $id, space_id: $space_id }}) 
                                ON CREATE
                                    SET n.`{attribute_id}` = $value
                                ON MATCH
                                    SET n.`{attribute_id}` = $value
                                "#,
                            ))
                            .param("id", self.entity_id.clone())
                            .param("value", value.clone())
                            .param("space_id", space_id),
                        )
                        .await?;
                }
            }
        };

        Ok(())
    }
}
