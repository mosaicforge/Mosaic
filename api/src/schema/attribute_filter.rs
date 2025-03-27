use juniper::GraphQLInputObject;

use grc20_core::mapping;

use super::triple::ValueType;

/// Filter the entities by attributes and their values and value types
#[derive(Debug, GraphQLInputObject)]
pub struct EntityAttributeFilter {
    pub attribute: String,

    pub value: Option<String>,
    pub value_not: Option<String>,
    pub value_in: Option<Vec<String>>,
    pub value_not_in: Option<Vec<String>>,

    pub value_type: Option<ValueType>,
    pub value_type_not: Option<ValueType>,
    pub value_type_in: Option<Vec<ValueType>>,
    pub value_type_not_in: Option<Vec<ValueType>>,
}

impl EntityAttributeFilter {
    fn value_filter(&self) -> mapping::PropFilter<String> {
        let mut filter = mapping::PropFilter::default();

        if let Some(value) = &self.value {
            filter = filter.value(value);
        }

        if let Some(value_not) = &self.value_not {
            filter = filter.value_not(value_not);
        }

        if let Some(value_in) = &self.value_in {
            filter = filter.value_in(value_in.clone());
        }

        if let Some(value_not_in) = &self.value_not_in {
            filter = filter.value_not_in(value_not_in.clone());
        }

        filter
    }

    fn value_type_filter(&self) -> mapping::PropFilter<String> {
        let mut filter = mapping::PropFilter::default();

        if let Some(value_type) = &self.value_type {
            filter = filter.value(value_type.to_string());
        }

        if let Some(value_type_not) = &self.value_type_not {
            filter = filter.value_not(value_type_not.to_string());
        }

        if let Some(value_type_in) = &self.value_type_in {
            filter = filter.value_in(
                value_type_in
                    .iter()
                    .map(|value_type| value_type.to_string())
                    .collect(),
            );
        }

        if let Some(value_type_not_in) = &self.value_type_not_in {
            filter = filter.value_not_in(
                value_type_not_in
                    .iter()
                    .map(|value_type| value_type.to_string())
                    .collect(),
            );
        }

        filter
    }
}

impl From<EntityAttributeFilter> for mapping::AttributeFilter {
    fn from(filter: EntityAttributeFilter) -> Self {
        mapping::AttributeFilter::new(&filter.attribute)
            .value(filter.value_filter())
            .value_type(filter.value_type_filter())
    }
}
