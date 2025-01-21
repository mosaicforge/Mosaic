use super::query_part::{IntoQueryPart, QueryPart};

pub struct ScalarFieldFilter {
    /// Unique identifier for the filter
    id: String,

    /// Variable name of the node on which the filter is applied
    node_var: String,

    /// Name of the field on which the filter is applied
    field_name: String,

    /// Value to use for equality predicate (e.g. `n.name = "test"`)
    value: Option<String>,

    /// Value to use for inequality predicate (e.g. `n.name <> "test"`)
    value_not: Option<String>,

    /// Values to use for `IN` predicate (e.g. `n.name IN ["test1", "test2"]`)
    value_in: Option<Vec<String>>,

    /// Values to use for `NOT IN` predicate (e.g. `n.name NOT IN ["test1", "test2"]`)
    value_not_in: Option<Vec<String>>,
}

impl ScalarFieldFilter {
    /// Create a new filter with a random ID
    pub fn new(node_var: &str, field_name: &str) -> Self {
        Self {
            id: format!("{}_{}", node_var, field_name.replace(".", "_")),
            node_var: node_var.to_owned(),
            field_name: field_name.to_owned(),
            value: None,
            value_in: None,
            value_not: None,
            value_not_in: None,
        }
    }

    /// Create a new filter with a specific ID
    pub fn with_id(node_var: &str, field_name: &str, id: &str) -> Self {
        Self {
            id: id.to_owned(),
            node_var: node_var.to_owned(),
            field_name: field_name.to_owned(),
            value: None,
            value_in: None,
            value_not: None,
            value_not_in: None,
        }
    }

    /// Set value to use for equality predicate (e.g. `n.name = "test"`)
    pub fn value(mut self, value: &str) -> Self {
        self.value = Some(value.to_owned());
        self
    }

    pub fn value_mut(&mut self, value: &str) {
        self.value = Some(value.to_owned());
    }

    /// Set value to use for inequality predicate (e.g. `n.name <> "test"`)
    pub fn value_not(mut self, value: &str) -> Self {
        self.value_not = Some(value.to_owned());
        self
    }

    pub fn value_not_mut(&mut self, value: &str) {
        self.value_not = Some(value.to_owned());
    }

    /// Set values to use for `IN` predicate (e.g. `n.name IN ["test1", "test2"]`)
    pub fn value_in(mut self, values: Vec<String>) -> Self {
        self.value_in = Some(values);
        self
    }

    pub fn value_in_mut(&mut self, values: Vec<String>) {
        self.value_in = Some(values);
    }

    /// Set values to use for `NOT IN` predicate (e.g. `n.name NOT IN ["test1", "test2"]`)
    pub fn value_not_in(mut self, values: Vec<String>) -> Self {
        self.value_not_in = Some(values);
        self
    }

    pub fn value_not_in_mut(&mut self, values: Vec<String>) {
        self.value_not_in = Some(values);
    }
}

impl IntoQueryPart for ScalarFieldFilter {
    fn into_query_part(self) -> QueryPart {
        let mut query_part = QueryPart::default();

        if let Some(value) = self.value {
            query_part = query_part.where_clause(&format!(
                "{}.`{}` = $value_{}",
                self.node_var, self.field_name, self.id,
            ));
            query_part = query_part.params(format!("value_{}", self.id), value.into());
        }

        if let Some(value_not) = self.value_not {
            query_part = query_part.where_clause(&format!(
                "{}.`{}` <> $value_not_{}",
                self.node_var, self.field_name, self.id,
            ));
            query_part = query_part.params(format!("value_not_{}", self.id), value_not.into());
        }

        if let Some(value_in) = self.value_in {
            query_part = query_part.where_clause(&format!(
                "{}.`{}` IN $value_in_{}",
                self.node_var, self.field_name, self.id,
            ));
            query_part = query_part.params(format!("value_in_{}", self.id), value_in.into());
        }

        if let Some(value_not_in) = self.value_not_in {
            query_part = query_part.where_clause(&format!(
                "{}.`{}` NOT IN $value_not_in_{}",
                self.node_var, self.field_name, self.id,
            ));
            query_part =
                query_part.params(format!("value_not_in_{}", self.id), value_not_in.into());
        }

        query_part
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_filter() {
        let query = ScalarFieldFilter::new("n", "name")
            .value("test")
            .value_not("test2")
            .value_in(vec!["test3".to_owned(), "test4".to_owned()])
            .value_not_in(vec!["test5".to_owned(), "test6".to_owned()])
            .into_query_part();

        assert_eq!(
            query,
            QueryPart {
                match_clauses: vec![],
                where_clauses: vec![
                    "n.`name` = $value_n_name".to_owned(),
                    "n.`name` <> $value_not_n_name".to_owned(),
                    "n.`name` IN $value_in_n_name".to_owned(),
                    "n.`name` NOT IN $value_not_in_n_name".to_owned(),
                ],
                return_clauses: Vec::new(),
                order_by_clauses: Vec::new(),
                params: HashMap::from([
                    ("value_n_name".to_owned(), "test".into()),
                    ("value_not_n_name".to_owned(), "test2".into()),
                    (
                        "value_in_n_name".to_owned(),
                        vec!["test3".to_owned(), "test4".to_owned()].into()
                    ),
                    (
                        "value_not_in_n_name".to_owned(),
                        vec!["test5".to_owned(), "test6".to_owned()].into()
                    ),
                ]),
            },
        )
    }
}
