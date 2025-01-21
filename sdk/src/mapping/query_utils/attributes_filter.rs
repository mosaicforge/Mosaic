use crate::mapping::query_utils::{
    query_part::{IntoQueryPart, QueryPart},
    scalar_filter::ScalarFieldFilter,
};

pub struct AttributeFilter {
    value_filter: ScalarFieldFilter,
    value_type_filter: ScalarFieldFilter,
}

impl AttributeFilter {
    pub fn new(node_var: &str, attribute: &str) -> Self {
        Self {
            value_filter: ScalarFieldFilter::new(node_var, attribute),
            value_type_filter: ScalarFieldFilter::new(node_var, &format!("{}.type", attribute)),
        }
    }

    pub fn with_id(node_var: &str, attribute: &str, id: &str) -> Self {
        Self {
            value_filter: ScalarFieldFilter::with_id(node_var, attribute, id),
            value_type_filter: ScalarFieldFilter::with_id(
                node_var,
                &format!("{}.type", attribute),
                &format!("vt_{id}"),
            ),
        }
    }

    pub fn value(mut self, value: &str) -> Self {
        self.value_filter = self.value_filter.value(value);
        self
    }

    pub fn value_mut(&mut self, value: &str) {
        self.value_filter.value_mut(value);
    }

    pub fn value_not(mut self, value: &str) -> Self {
        self.value_filter = self.value_filter.value_not(value);
        self
    }

    pub fn value_not_mut(&mut self, value: &str) {
        self.value_filter.value_not_mut(value);
    }

    pub fn value_in(mut self, values: Vec<String>) -> Self {
        self.value_filter = self.value_filter.value_in(values);
        self
    }

    pub fn value_in_mut(&mut self, values: Vec<String>) {
        self.value_filter.value_in_mut(values);
    }

    pub fn value_not_in(mut self, values: Vec<String>) -> Self {
        self.value_filter = self.value_filter.value_not_in(values);
        self
    }

    pub fn value_not_in_mut(&mut self, values: Vec<String>) {
        self.value_filter.value_not_in_mut(values);
    }

    pub fn value_type(mut self, value: &str) -> Self {
        self.value_type_filter = self.value_type_filter.value(value);
        self
    }

    pub fn value_type_mut(&mut self, value: &str) {
        self.value_type_filter.value_mut(value);
    }

    pub fn value_type_not(mut self, value: &str) -> Self {
        self.value_type_filter = self.value_type_filter.value_not(value);
        self
    }

    pub fn value_type_not_mut(&mut self, value: &str) {
        self.value_type_filter.value_not_mut(value);
    }

    pub fn value_type_in(mut self, values: Vec<String>) -> Self {
        self.value_type_filter = self.value_type_filter.value_in(values);
        self
    }

    pub fn value_type_in_mut(&mut self, values: Vec<String>) {
        self.value_type_filter.value_in_mut(values);
    }

    pub fn value_type_not_in(mut self, values: Vec<String>) -> Self {
        self.value_type_filter = self.value_type_filter.value_not_in(values);
        self
    }

    pub fn value_type_not_in_mut(&mut self, values: Vec<String>) {
        self.value_type_filter.value_not_in_mut(values);
    }
}

impl IntoQueryPart for AttributeFilter {
    fn into_query_part(self) -> QueryPart {
        let query_parts = vec![
            self.value_filter.into_query_part(),
            self.value_type_filter.into_query_part(),
        ];

        query_parts
            .into_iter()
            .fold(QueryPart::default(), |acc, part| acc.merge(part))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    #[test]
    fn test_attribute_filter() {
        let filter = AttributeFilter::new("n", "name")
            .value_in(vec!["test".to_string(), "test2".to_string()])
            .value_type("TEXT");

        let query_part = filter.into_query_part();

        assert_eq!(
            query_part,
            QueryPart {
                where_clauses: vec![
                    "n.`name` IN $value_in_n_name".to_string(),
                    "n.`name.type` = $value_n_name_type".to_string(),
                ],
                params: HashMap::from([
                    (
                        "value_in_n_name".to_string(),
                        vec!["test".to_string(), "test2".to_string()].into()
                    ),
                    ("value_n_name_type".to_string(), "TEXT".to_string().into())
                ]),
                ..QueryPart::default()
            }
        );

        assert_eq!(
            query_part.query(),
            "WHERE n.`name` IN $value_in_n_name\nAND n.`name.type` = $value_n_name_type\n"
        );
    }
}
