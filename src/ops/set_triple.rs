use crate::{
    kg::client::{AttributeLabel, RelationLabel, TypeLabel},
    system_ids,
};

use crate::ops::{KgOp, Value};

pub struct SetTriple {
    pub entity_id: String,
    pub attribute_id: String,
    pub value: Value,
}

impl KgOp for SetTriple {
    async fn apply_op(&self, kg: &crate::kg::client::Client) -> anyhow::Result<()> {
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

        match (&self.value, self.attribute_id.as_str()) {
            (Value::Entity(value), system_ids::TYPES) => {
                let type_label = kg
                    .get_name(&value)
                    .await?
                    .map_or(TypeLabel::new(&value), |name| TypeLabel::new(&name));

                kg.neo4j
                    .run(
                        neo4rs::query(&format!(
                            r#"
                            MERGE (t {{ id: $value }})
                            MERGE (n {{ id: $id }}) 
                            ON CREATE 
                                SET n :{type_label}
                            ON MATCH 
                                SET n :{type_label}
                            "#,
                            // MERGE (n) -[:TYPE {{id: $attribute_id}}]-> (t)
                            // "#,
                        ))
                        .param("id", self.entity_id.clone())
                        .param("value", self.value.clone()), // .param("attribute_id", self.attribute_id.clone()),
                    )
                    .await?;
            }
            (Value::Text(value), system_ids::NAME) => {
                kg.set_name(&self.entity_id, &value).await?;
            }
            (Value::Entity(value), attribute_id) => {
                kg.neo4j
                    .run(
                        neo4rs::query(&format!(
                            r#"
                            MERGE (n {{ id: $id }})
                            MERGE (m {{ id: $value }})
                            MERGE (n) -[:{relation_label} {{id: $attribute_id}}]-> (m)
                            "#,
                            relation_label = RelationLabel::new(&attribute_name),
                        ))
                        .param("id", self.entity_id.clone())
                        .param("value", value.clone())
                        .param("attribute_id", attribute_id),
                    )
                    .await?;
            }
            (value, _) => {
                kg.neo4j
                    .run(
                        neo4rs::query(&format!(
                            r#"
                            MERGE (n {{ id: $id }}) 
                            ON CREATE
                                SET n.{attribute_name} = $value
                            ON MATCH
                                SET n.{attribute_name} = $value
                            "#,
                            attribute_name = AttributeLabel::new(&attribute_name),
                        ))
                        .param("id", self.entity_id.clone())
                        .param("value", value.clone()),
                    )
                    .await?;
            }
        };

        Ok(())
    }
}
