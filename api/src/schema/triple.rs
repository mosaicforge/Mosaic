use std::fmt::Display;

use juniper::{graphql_object, Executor, GraphQLEnum, GraphQLObject, ScalarValue};

use grc20_core::{
    mapping::{self, query_utils::Query, triple},
    system_ids,
};

use crate::context::KnowledgeGraph;

#[derive(Debug)]
pub struct Triple {
    pub attribute: String,
    pub value: String,
    pub value_type: ValueType,
    pub options: Options,

    pub space_id: String,
    pub space_version: Option<String>,
}

impl Triple {
    pub fn new(triple: mapping::Triple, space_id: String, space_version: Option<String>) -> Self {
        Self {
            attribute: triple.attribute,
            value: triple.value.value,
            value_type: triple.value.value_type.into(),
            options: triple.value.options.into(),

            space_id,
            space_version,
        }
    }
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

    /// Space ID of the triple
    fn space_id(&self) -> &str {
        &self.space_id
    }

    /// Name of the attribute (if available)
    async fn name<'a, S: ScalarValue>(
        &'a self,
        executor: &'a Executor<'_, '_, KnowledgeGraph, S>,
    ) -> Option<String> {
        triple::find_one(
            &executor.context().0,
            system_ids::NAME_ATTRIBUTE,
            &self.attribute,
            &self.space_id,
            self.space_version.clone(),
        )
        .send()
        .await
        .expect("Failed to find triple name attribute")
        .map(|triple| triple.value.value)
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

#[derive(Debug, GraphQLObject)]
pub struct Options {
    pub format: Option<String>,
    pub unit: Option<String>,
    pub language: Option<String>,
}

impl From<mapping::Options> for Options {
    fn from(options: mapping::Options) -> Self {
        Self {
            format: options.format,
            unit: options.unit,
            language: options.language,
        }
    }
}

#[derive(Debug, GraphQLEnum, PartialEq)]
pub enum ValueType {
    Text,
    Number,
    Checkbox,
    Url,
    Time,
    Point,
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Text => write!(f, "TEXT"),
            ValueType::Number => write!(f, "NUMBER"),
            ValueType::Checkbox => write!(f, "CHECKBOX"),
            ValueType::Url => write!(f, "URL"),
            ValueType::Time => write!(f, "TIME"),
            ValueType::Point => write!(f, "POINT"),
        }
    }
}