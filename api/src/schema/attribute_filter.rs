use juniper::GraphQLInputObject;

use sdk::mapping;

use crate::schema::ValueType;

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
    pub fn add_to_entity_query(
        self,
        mut query: mapping::entity_queries::FindMany,
    ) -> mapping::entity_queries::FindMany {
        if let Some(value) = self.value {
            query = query.attribute(&self.attribute, &value);
        }

        if let Some(value_not) = self.value_not {
            query = query.attribute_not(&self.attribute, &value_not);
        }

        if let Some(value_in) = self.value_in {
            query = query.attribute_in(&self.attribute, value_in.clone());
        }

        if let Some(value_not_in) = self.value_not_in {
            query = query.attribute_not_in(&self.attribute, value_not_in.clone());
        }

        if let Some(value_type) = self.value_type {
            query = query.attribute_value_type(&self.attribute, &value_type.to_string());
        }

        if let Some(value_type_not) = self.value_type_not {
            query = query.attribute_value_type_not(&self.attribute, &value_type_not.to_string());
        }

        if let Some(value_type_in) = self.value_type_in {
            query = query.attribute_value_type_in(
                &self.attribute,
                value_type_in.into_iter().map(|vt| vt.to_string()).collect(),
            );
        }

        if let Some(value_type_not_in) = self.value_type_not_in {
            query = query.attribute_value_type_not_in(
                &self.attribute,
                value_type_not_in
                    .into_iter()
                    .map(|vt| vt.to_string())
                    .collect(),
            );
        }

        query
    }

    pub fn add_to_relation_query(
        self,
        mut query: mapping::relation_queries::FindMany,
    ) -> mapping::relation_queries::FindMany {
        if let Some(value) = self.value {
            query = query.attribute(&self.attribute, &value);
        }

        if let Some(value_not) = self.value_not {
            query = query.attribute_not(&self.attribute, &value_not);
        }

        if let Some(value_in) = self.value_in {
            query = query.attribute_in(&self.attribute, value_in.clone());
        }

        if let Some(value_not_in) = self.value_not_in {
            query = query.attribute_not_in(&self.attribute, value_not_in.clone());
        }

        if let Some(value_type) = self.value_type {
            query = query.attribute_value_type(&self.attribute, &value_type.to_string());
        }

        if let Some(value_type_not) = self.value_type_not {
            query = query.attribute_value_type_not(&self.attribute, &value_type_not.to_string());
        }

        if let Some(value_type_in) = self.value_type_in {
            query = query.attribute_value_type_in(
                &self.attribute,
                value_type_in.into_iter().map(|vt| vt.to_string()).collect(),
            );
        }

        if let Some(value_type_not_in) = self.value_type_not_in {
            query = query.attribute_value_type_not_in(
                &self.attribute,
                value_type_not_in
                    .into_iter()
                    .map(|vt| vt.to_string())
                    .collect(),
            );
        }

        query
    }
}
