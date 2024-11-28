use kg_core::{pb::grc20, system_ids};

use super::KgOp;

pub struct CreateRelation {
    /// ID of the relation entity
    pub entity_id: String,
    /// ID of the "from" entity
    pub from_entity_id: String,
    /// ID of the "to" entity
    pub to_entity_id: String,
    /// ID of the relation type entity
    pub relation_type_id: String,
    /// Index of the relation
    pub index: String,
}

impl KgOp for CreateRelation {
    async fn apply_op(&self, kg: &crate::kg::Client, space_id: &str) -> anyhow::Result<()> {
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
                let type_label = match kg.get_name(&self.to_entity_id).await? {
                    Some(name) if name.replace(" ", "").is_empty() => self.to_entity_id.clone(),
                    Some(name) => name,
                    None => self.to_entity_id.clone(),
                };

                tracing::info!(
                    "SetType {}: {}",
                    self.from_entity_id,
                    if type_label == self.to_entity_id {
                        self.to_entity_id.to_string()
                    } else {
                        format!("{} ({})", type_label, self.to_entity_id)
                    },
                );

                kg.neo4j
                    .run(
                        neo4rs::query(&format!(
                            r#"
                            MERGE (n {{ id: $id, space_id: $space_id }}) 
                            ON CREATE 
                                SET n :`{type_id}`
                            ON MATCH 
                                SET n :`{type_id}`
                            "#,
                            type_id = self.to_entity_id
                        ))
                        .param("id", self.from_entity_id.clone())
                        .param("space_id", space_id),
                    )
                    .await?;
            }
            _ => {
                kg.neo4j
                    .run(
                        neo4rs::query(&format!(
                            r#"
                            MERGE (from {{id: $from_id, space_id: $space_id}})
                            MERGE (to {{id: $to_id, space_id: $space_id}})
                            MERGE (from)-[:`{relation_type_id}` {{id: $relation_id, `{index_id}`: $index, space_id: $space_id}}]->(to)
                            "#,
                            relation_type_id = self.relation_type_id,
                            index_id = system_ids::RELATION_INDEX
                        ))
                        .param("from_id", self.from_entity_id.clone())
                        .param("to_id", self.to_entity_id.clone())
                        .param("relation_id", self.entity_id.clone())
                        .param("index", self.index.clone())
                        .param("relation_type_id", self.relation_type_id.clone())
                        .param("space_id", space_id)
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
    pub fn from_ops(mut self, ops: &[grc20::Op]) -> (Self, Vec<&grc20::Op>) {
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
                    && *r#type == grc20::ValueType::Entity as i32
                    && !value.is_empty() =>
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
            from_entity_id: match self.from_entity_id {
                Some(id) if id.is_empty() => {
                    return Err(anyhow::anyhow!(
                        "{}: Invalid from entity id: `{id}`",
                        self.entity_id
                    ))
                }
                Some(id) => id,
                None => return Err(anyhow::anyhow!("{}: Missing from entity", self.entity_id)),
            },
            to_entity_id: match self.to_entity_id {
                Some(id) if id.is_empty() => {
                    return Err(anyhow::anyhow!(
                        "{}: Invalid to entity id: `{id}`",
                        self.entity_id
                    ))
                }
                Some(id) => id,
                None => return Err(anyhow::anyhow!("{}: Missing to entity", self.entity_id)),
            },
            relation_type_id: match self.relation_type_id {
                Some(id) if id.is_empty() => {
                    return Err(anyhow::anyhow!(
                        "{}: Invalid relation type id: `{id}`",
                        self.entity_id
                    ))
                }
                Some(id) => id,
                None => return Err(anyhow::anyhow!("{}: Missing relation type", self.entity_id)),
            },
            // relation_type_id: match self.relation_type_id {
            //     Some(id) if id.is_empty() => {
            //         tracing::warn!("{}: Invalid relation type id: `{id}`! Using default _UNKNOWN", self.entity_id);
            //         "_UNKNOWN".to_string()
            //     },
            //     Some(id) => id,
            //     None => {
            //         tracing::warn!("{}: Missing relation type! Using default _UNKNOWN", self.entity_id);
            //         "_UNKNOWN".to_string()
            //     },
            // },
            index: self.index.unwrap_or_else(|| "a0".to_string()),
            entity_id: self.entity_id,
        })
    }
}
