use crate::{
    grc20,
    kg::client::{RelationLabel, TypeLabel},
    system_ids,
};

use super::KgOp;

pub struct CreateRelation {
    pub entity_id: String,
    pub from_entity_id: String,
    pub to_entity_id: String,
    pub relation_type_id: String,
    pub index: String,
}

impl KgOp for CreateRelation {
    async fn apply_op(&self, kg: &crate::kg::Client) -> anyhow::Result<()> {
        let relation_name = kg
            .get_name(&self.relation_type_id)
            .await?
            .unwrap_or(self.relation_type_id.to_string());

        tracing::info!(
            "CreateRelation {}: {} {} -> {}",
            self.entity_id,
            if relation_name == self.relation_type_id {
                self.relation_type_id.to_string()
            } else {
                format!("{} ({})", relation_name, self.relation_type_id)
            },
            self.from_entity_id,
            self.to_entity_id,
        );

        match self.relation_type_id.as_str() {
            system_ids::TYPES => {
                let type_label = kg
                    .get_name(&self.to_entity_id)
                    .await?
                    .map_or(TypeLabel::new(&self.to_entity_id), |name| {
                        TypeLabel::new(&name)
                    });

                kg.neo4j
                    .run(
                        neo4rs::query(&format!(
                            r#"
                            MERGE (n {{ id: $id }}) 
                            ON CREATE 
                                SET n :{type_label}
                            ON MATCH 
                                SET n :{type_label}
                            "#,
                            // MERGE (n) -[:TYPE {{id: $attribute_id}}]-> (t)
                            // "#,
                        ))
                        .param("id", self.from_entity_id.clone()),
                    )
                    .await?;
            }
            _ => {
                kg.neo4j
                    .run(
                        neo4rs::query(&format!(
                            r#"
                        MERGE (from {{id: $from_id}})
                        MERGE (to {{id: $to_id}})
                        MERGE (from)-[:{relation_label} {{id: $relation_id, index: $index}}]->(to)
                        "#,
                            relation_label = RelationLabel::new(&relation_name)
                        ))
                        .param("from_id", self.from_entity_id.clone())
                        .param("to_id", self.to_entity_id.clone())
                        .param("relation_id", self.entity_id.clone())
                        .param("index", self.index.clone()),
                    )
                    .await?;
            }
        }

        Ok(())
    }
}

pub struct CreateRelationBuilder {
    entity_id: String,
    from_entity_id: Option<String>,
    to_entity_id: Option<String>,
    relation_type_id: Option<String>,
    index: Option<String>,
}

impl CreateRelationBuilder {
    pub fn new(entity_id: String) -> Self {
        CreateRelationBuilder {
            entity_id,
            from_entity_id: None,
            to_entity_id: None,
            relation_type_id: None,
            index: None,
        }
    }

    /// Extracts the from, to, and relation type entities from the ops and returns the remaining ops
    pub fn from_ops<'a>(mut self, ops: &'a Vec<grc20::Op>) -> (Self, Vec<&'a grc20::Op>) {
        let remaining = ops
            .iter()
            .filter(|op| match (op.r#type(), &op.triple) {
                // Ignore the TYPES relation of the entity
                (
                    grc20::OpType::SetTriple,
                    Some(grc20::Triple {
                        attribute,
                        value: Some(grc20::Value { r#type, .. }),
                        ..
                    }),
                ) if attribute == system_ids::TYPES
                    && *r#type == grc20::ValueType::Entity as i32 =>
                {
                    false
                }

                // Set the FROM_ENTITY attribute
                (
                    grc20::OpType::SetTriple,
                    Some(grc20::Triple {
                        attribute,
                        value: Some(grc20::Value { r#type, value }),
                        ..
                    }),
                ) if attribute == system_ids::RELATION_FROM_ATTRIBUTE
                    && *r#type == grc20::ValueType::Entity as i32 =>
                {
                    self.from_entity_id = Some(value.clone());
                    false
                }

                // Set the TO_ENTITY attribute
                (
                    grc20::OpType::SetTriple,
                    Some(grc20::Triple {
                        attribute,
                        value: Some(grc20::Value { r#type, value }),
                        ..
                    }),
                ) if attribute == system_ids::RELATION_TO_ATTRIBUTE
                    && *r#type == grc20::ValueType::Entity as i32 =>
                {
                    self.to_entity_id = Some(value.clone());
                    false
                }

                // Set the RELATION_TYPE attribute
                (
                    grc20::OpType::SetTriple,
                    Some(grc20::Triple {
                        attribute,
                        value: Some(grc20::Value { r#type, value }),
                        ..
                    }),
                ) if attribute == system_ids::RELATION_TYPE_ATTRIBUTE
                    && *r#type == grc20::ValueType::Entity as i32 =>
                {
                    self.relation_type_id = Some(value.clone());
                    false
                }

                // Set the INDEX attribute
                (
                    grc20::OpType::SetTriple,
                    Some(grc20::Triple {
                        attribute,
                        value: Some(grc20::Value { r#type, value }),
                        ..
                    }),
                ) if attribute == system_ids::RELATION_INDEX
                    && *r#type == grc20::ValueType::Text as i32 =>
                {
                    self.index = Some(value.clone());
                    false
                }

                _ => true,
            })
            .collect();

        (self, remaining)
    }

    pub fn build(self) -> anyhow::Result<CreateRelation> {
        Ok(CreateRelation {
            entity_id: self.entity_id,
            from_entity_id: self
                .from_entity_id
                .ok_or_else(|| anyhow::anyhow!("Missing from entity"))?,
            to_entity_id: self
                .to_entity_id
                .ok_or_else(|| anyhow::anyhow!("Missing to entity"))?,
            relation_type_id: self
                .relation_type_id
                .ok_or_else(|| anyhow::anyhow!("Missing relation type"))?,
            index: self.index.unwrap_or_else(|| "a0".to_string()),
        })
    }
}
