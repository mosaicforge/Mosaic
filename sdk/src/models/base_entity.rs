use crate::{error::DatabaseError, mapping::{self, EntityNode}, system_ids};

#[derive(Clone, Debug)]
pub struct BaseEntity {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cover: Option<String>,
}

impl BaseEntity {
    pub async fn types(&self, neo4j: &neo4rs::Graph, space_id: impl Into<String>, space_version: Option<i64>) -> Result<Vec<EntityNode>, DatabaseError> {
        todo!()
    }

    pub async fn blocks(&self, neo4j: &neo4rs::Graph, space_id: impl Into<String>, space_version: Option<i64>) -> Result<Vec<EntityNode>, DatabaseError> {
        todo!()
    }
}

impl mapping::FromAttributes for BaseEntity {
    fn from_attributes(mut attributes: mapping::Attributes) -> Result<Self, mapping::TriplesConversionError> {
        Ok(Self {
            name: attributes.pop_opt(system_ids::NAME_ATTRIBUTE)?,
            description: attributes.pop_opt(system_ids::DESCRIPTION_ATTRIBUTE)?,
            cover: attributes.pop_opt(system_ids::COVER_ATTRIBUTE)?,
        })
    }
}