use super::query_part::{IntoQueryPart, QueryPart};

pub struct ListFieldFilter {
    /// Unique identifier for the filter
    id: String,

    /// Variable name of the node on which the filter is applied
    node_var: String,

    /// Name of the field on which the filter is applied
    field_name: String,

    /// Value to use for equality predicate (e.g. `n.foos = ["test"]`)
    value: Option<Vec<String>>,

    /// Value to use for inequality predicate (e.g. `n.foos <> ["test"]`)
    value_not: Option<Vec<String>>,

    /// Values to use for `ALL ... IN` predicate (e.g. `ALL(x IN ["test1", "test2"] WHERE x IN n.foos)`)
    value_contains: Option<Vec<String>>,

    /// Values to use for `ALL .. NOT IN` predicate (e.g. `ALL(x IN ["test1", "test2"] WHERE x NOT IN n.foos)`)
    value_not_contains: Option<Vec<String>>,
}

impl ListFieldFilter {
    /// Create a new filter with a random ID
    pub fn new(node_var: &str, field_name: &str) -> Self {
        Self {
            id: format!("{}_{}", node_var, field_name.replace(".", "_")),
            node_var: node_var.to_owned(),
            field_name: field_name.to_owned(),
            value: None,
            value_contains: None,
            value_not: None,
            value_not_contains: None,
        }
    }

    /// Create a new filter with a specific ID
    pub fn with_id(node_var: &str, field_name: &str, id: &str) -> Self {
        Self {
            id: id.to_owned(),
            node_var: node_var.to_owned(),
            field_name: field_name.to_owned(),
            value: None,
            value_contains: None,
            value_not: None,
            value_not_contains: None,
        }
    }

    /// Set value to use for equality predicate (e.g. `n.name = "test"`)
    pub fn value(mut self, values: Vec<String>) -> Self {
        self.value = Some(values);
        self
    }

    pub fn value_mut(&mut self, values: Vec<String>) {
        self.value = Some(values);
    }

    /// Set value to use for inequality predicate (e.g. `n.name <> "test"`)
    pub fn value_not(mut self, values: Vec<String>) -> Self {
        self.value_not = Some(values);
        self
    }

    pub fn value_not_mut(&mut self, values: Vec<String>) {
        self.value_not = Some(values);
    }

    /// Set values to use for `ALL ... IN` predicate (e.g. `ALL(x IN ["test1", "test2"] WHERE x IN n.foos)`)
    pub fn value_contains(mut self, values: Vec<String>) -> Self {
        self.value_contains = Some(values);
        self
    }

    pub fn value_contains_mut(&mut self, values: Vec<String>) {
        self.value_contains = Some(values);
    }

    /// Set values to use for `ALL .. NOT IN` predicate (e.g. `ALL(x IN ["test1", "test2"] WHERE x NOT IN n.foos)`)
    pub fn value_not_contains(mut self, values: Vec<String>) -> Self {
        self.value_not_contains = Some(values);
        self
    }

    pub fn value_not_contains_mut(&mut self, values: Vec<String>) {
        self.value_not_contains = Some(values);
    }
}

impl IntoQueryPart for ListFieldFilter {
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

        if let Some(value_contains) = self.value_contains {
            query_part = query_part.where_clause(&format!(
                "ALL(x IN $value_contains_{} WHERE x IN {}.`{}`)",
                self.id, self.node_var, self.field_name,
            ));
            query_part =
                query_part.params(format!("value_contains_{}", self.id), value_contains.into());
        }

        if let Some(value_not_contains) = self.value_not_contains {
            query_part = query_part.where_clause(&format!(
                "NOT ANY(x IN $value_not_contains_{} WHERE x IN {}.`{}`)",
                self.id, self.node_var, self.field_name,
            ));
            query_part = query_part.params(
                format!("value_not_contains_{}", self.id),
                value_not_contains.into(),
            );
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
        let query = ListFieldFilter::new("n", "name")
            .value(vec!["test".to_owned()])
            .value_not(vec!["test2".to_owned()])
            .value_contains(vec!["test3".to_owned(), "test4".to_owned()])
            .value_not_contains(vec!["test5".to_owned(), "test6".to_owned()])
            .into_query_part();

        assert_eq!(
            query,
            QueryPart {
                match_clauses: vec![],
                where_clauses: vec![
                    "n.`name` = $value_n_name".to_owned(),
                    "n.`name` <> $value_not_n_name".to_owned(),
                    "ALL(x IN $value_contains_n_name WHERE x IN n.`name`)".to_owned(),
                    "NOT ANY(x IN $value_not_contains_n_name WHERE x IN n.`name`)".to_owned(),
                ],
                return_clauses: Vec::new(),
                order_by_clauses: Vec::new(),
                params: HashMap::from([
                    ("value_n_name".to_owned(), vec!["test".to_owned()].into()),
                    (
                        "value_not_n_name".to_owned(),
                        vec!["test2".to_owned()].into()
                    ),
                    (
                        "value_contains_n_name".to_owned(),
                        vec!["test3".to_owned(), "test4".to_owned()].into()
                    ),
                    (
                        "value_not_contains_n_name".to_owned(),
                        vec!["test5".to_owned(), "test6".to_owned()].into()
                    ),
                ]),
                ..Default::default()
            },
        )
    }
}
