use crate::kg::client::{AttributeLabel, Entity};

use crate::ops::KgOp;

pub struct DeleteTriple {
    pub entity_id: String,
    pub attribute_id: String,
}

impl KgOp for DeleteTriple {
    async fn apply_op(&self, kg: &crate::kg::client::Client) -> anyhow::Result<()> {
        let entity_name = kg
            .find_one::<Entity>(Entity::find_by_id_query(&self.entity_id))
            .await?
            .and_then(|entity| entity.name)
            .unwrap_or(self.entity_id.to_string());

        let attribute_name = kg
            .get_name(&self.attribute_id)
            .await?
            .unwrap_or(self.attribute_id.to_string());

        tracing::info!(
            "DeleteTriple: {}, {}",
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
        );

        kg.neo4j
            .run(
                neo4rs::query(&format!(
                    r#"
                    MATCH (n {{ id: $id }})
                    REMOVE n.{attribute_name}
                    "#,
                    attribute_name = AttributeLabel::new(&attribute_name),
                ))
                .param("id", self.entity_id.clone()),
            )
            .await?;

        Ok(())
    }
}
