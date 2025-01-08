use juniper::{graphql_object, Executor, ScalarValue};

use sdk::mapping;

use crate::context::KnowledgeGraph;

use super::{Options, ValueType};

#[derive(Debug)]
pub struct Triple {
    pub(crate) space_id: String,
    pub(crate) attribute: String,
    pub(crate) value: String,
    pub(crate) value_type: ValueType,
    pub(crate) options: Options,
}

#[graphql_object]
#[graphql(context = KnowledgeGraph, scalar = S: ScalarValue)]
impl Triple {
    /// Attribute ID of the triple
    fn attribute(&self) -> &str {
        &self.attribute
    }

    /// Value of the triple
    fn value(&self) -> &str {
        &self.value
    }

    /// Value type of the triple
    fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    /// Options of the triple (if any)
    fn options(&self) -> &Options {
        &self.options
    }

    /// Name of the attribute (if available)
    async fn name<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Option<String> {
        mapping::Entity::<mapping::Named>::find_by_id(
            &executor.context().0,
            &self.attribute,
            &self.space_id,
        )
        .await
        .expect("Failed to find attribute entity")
        .and_then(|entity| entity.name())
    }
}

impl From<mapping::ValueType> for ValueType {
    fn from(value_type: mapping::ValueType) -> Self {
        match value_type {
            mapping::ValueType::Text => Self::Text,
            mapping::ValueType::Number => Self::Number,
            mapping::ValueType::Checkbox => Self::Checkbox,
            mapping::ValueType::Url => Self::Url,
            mapping::ValueType::Time => Self::Time,
            mapping::ValueType::Point => Self::Point,
        }
    }
}

impl From<ValueType> for mapping::ValueType {
    fn from(value_type: ValueType) -> Self {
        match value_type {
            ValueType::Text => mapping::ValueType::Text,
            ValueType::Number => mapping::ValueType::Number,
            ValueType::Checkbox => mapping::ValueType::Checkbox,
            ValueType::Url => mapping::ValueType::Url,
            ValueType::Time => mapping::ValueType::Time,
            ValueType::Point => mapping::ValueType::Point,
        }
    }
}
