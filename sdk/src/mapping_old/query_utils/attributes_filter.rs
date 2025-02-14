use super::query_part::{IntoQueryPart, QueryPart};

pub struct AttributeFilter {
    /// Unique identifier for the filter
    id: String,

    /// Variable name of the node on which the filter is applied
    node_var: String,

    /// Name of the field on which the filter is applied
    field_name: String,

    /// Space ID
    space_id: Option<String>,

    /// Version index
    version: Option<i64>,

    /// Value to use for equality predicate (e.g. `n.name = "test"`)
    value: Option<String>,

    /// Value to use for inequality predicate (e.g. `n.name <> "test"`)
    value_not: Option<String>,

    /// Values to use for `IN` predicate (e.g. `n.name IN ["test1", "test2"]`)
    value_in: Option<Vec<String>>,

    /// Values to use for `NOT IN` predicate (e.g. `n.name NOT IN ["test1", "test2"]`)
    value_not_in: Option<Vec<String>>,

    /// Value to use for value type equality predicate (e.g. `n.name = "test"`)
    value_type: Option<String>,

    /// Value to use for value type inequality predicate (e.g. `n.name <> "test"`)
    value_type_not: Option<String>,

    /// Values to use for value type `IN` predicate (e.g. `n.name IN ["test1", "test2"]`)
    value_type_in: Option<Vec<String>>,

    /// Values to use for value type `NOT IN` predicate (e.g. `n.name NOT IN ["test1", "test2"]`)
    value_type_not_in: Option<Vec<String>>,
}

impl AttributeFilter {
    /// Create a new filter with a random ID
    pub fn new(node_var: &str, field_name: &str) -> Self {
        Self {
            id: format!("{}_{}", node_var, field_name.replace(".", "_")),
            node_var: node_var.to_owned(),
            field_name: field_name.to_owned(),
            space_id: None,
            version: None,
            value: None,
            value_in: None,
            value_not: None,
            value_not_in: None,
            value_type: None,
            value_type_in: None,
            value_type_not: None,
            value_type_not_in: None,            
        }
    }

    /// Create a new filter with a specific ID
    pub fn with_id(node_var: &str, field_name: &str, id: &str) -> Self {
        Self {
            id: id.to_owned(),
            node_var: node_var.to_owned(),
            field_name: field_name.to_owned(),
            space_id: None,
            version: None,
            value: None,
            value_in: None,
            value_not: None,
            value_not_in: None,
            value_type: None,
            value_type_in: None,
            value_type_not: None,
            value_type_not_in: None,
        }
    }

    pub fn space_id(mut self, space_id: &str) -> Self {
        self.space_id = Some(space_id.to_owned());
        self
    }

    pub fn space_id_mut(&mut self, space_id: &str) {
        self.space_id = Some(space_id.to_owned());
    }

    pub fn space_id_opt(mut self, space_id: Option<String>) -> Self {
        self.space_id = space_id;
        self
    }

    pub fn space_id_opt_mut(&mut self, space_id: Option<String>) {
        self.space_id = space_id;
    }

    pub fn version(mut self, version_index: i64) -> Self {
        self.version = Some(version_index);
        self
    }

    pub fn version_mut(&mut self, version_index: i64) {
        self.version = Some(version_index);
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

    pub fn value_type(mut self, value: &str) -> Self {
        self.value_type = Some(value.to_owned());
        self
    }

    pub fn value_type_mut(&mut self, value: &str) {
        self.value_type = Some(value.to_owned());
    }

    pub fn value_type_not(mut self, value: &str) -> Self {
        self.value_type_not = Some(value.to_owned());
        self
    }

    pub fn value_type_not_mut(&mut self, value: &str) {
        self.value_type_not = Some(value.to_owned());
    }

    pub fn value_type_in(mut self, values: Vec<String>) -> Self {
        self.value_type_in = Some(values);
        self
    }

    pub fn value_type_in_mut(&mut self, values: Vec<String>) {
        self.value_type_in = Some(values);
    }

    pub fn value_type_not_in(mut self, values: Vec<String>) -> Self {
        self.value_type_not_in = Some(values);
        self
    }

    pub fn value_type_not_in_mut(&mut self, values: Vec<String>) {
        self.value_type_not_in = Some(values);
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_none()
            && self.value_not.is_none()
            && self.value_in.is_none()
            && self.value_not_in.is_none()
            && self.value_type.is_none()
            && self.value_type_not.is_none()
            && self.value_type_in.is_none()
            && self.value_type_not_in.is_none()
    }
}

impl IntoQueryPart for AttributeFilter {
    fn into_query_part(self) -> QueryPart {
        let mut query_part = QueryPart::default();

        if !self.is_empty() {
            query_part = query_part.match_clause(&format!(
                r#"({node_var}) -[r_{filter_id}:ATTRIBUTE]-> (val_{filter_id} {{attribute: "{attribute_id}"}})"#,
                node_var=self.node_var, 
                attribute_id=self.field_name, 
                filter_id=self.id,
            ));

            if self.space_id.is_some() {
                query_part = query_part.where_clause(&format!(
                    "r_{filter_id}.space_id = $space_id",
                    filter_id=self.id,
                ));
                query_part = query_part.params("space_id".to_owned(), self.space_id.unwrap().into());
            }

            if self.version.is_some() {
                // If version index is set, we need to check if the version is within the range
                query_part = query_part.where_clause(&format!(
                    "r_{filter_id}.min_version =< $version AND (r_{filter_id}.max_version > $version OR r_{filter_id}.max_version IS NULL)",
                    filter_id=self.id,
                ));
            } else {
                // If version index is not set, we assume current version (i.e.: max_version is NULL)
                query_part = query_part.where_clause(&format!(
                    "r_{filter_id}.max_version IS NULL",
                    filter_id=self.id,
                ));
            }
        }

        if let Some(value) = self.value {
            query_part = query_part.where_clause(&format!(
                "val_{filter_id}.value = $value_{filter_id}",
                filter_id=self.id,
            ));
            query_part = query_part.params(format!("value_{}", self.id), value.into());
        }

        if let Some(value_not) = self.value_not {
            query_part = query_part.where_clause(&format!(
                "val_{filter_id}.value <> $value_not_{filter_id}",
                filter_id=self.id,
            ));
            query_part = query_part.params(format!("value_not_{}", self.id), value_not.into());
        }

        if let Some(value_in) = self.value_in {
            query_part = query_part.where_clause(&format!(
                "val_{filter_id}.value IN $value_in_{filter_id}",
                filter_id=self.id,
            ));
            query_part = query_part.params(format!("value_in_{}", self.id), value_in.into());
        }

        if let Some(value_not_in) = self.value_not_in {
            query_part = query_part.where_clause(&format!(
                "val_{filter_id}.value NOT IN $value_not_in_{filter_id}",
                filter_id=self.id,
            ));
            query_part =
                query_part.params(format!("value_not_in_{}", self.id), value_not_in.into());
        }

        if let Some(value_type) = self.value_type {
            query_part = query_part.where_clause(&format!(
                "val_{filter_id}.value_type = $value_type_{filter_id}",
                filter_id=self.id,
            ));
            query_part = query_part.params(format!("value_type_{}", self.id), value_type.into());
        }

        if let Some(value_type_not) = self.value_type_not {
            query_part = query_part.where_clause(&format!(
                "val_{filter_id}.value_type <> $value_type_not_{filter_id}",
                filter_id=self.id,
            ));
            query_part = query_part.params(format!("value_type_not_{}", self.id), value_type_not.into());
        }

        if let Some(value_type_in) = self.value_type_in {
            query_part = query_part.where_clause(&format!(
                "val_{filter_id}.value_type IN $value_type_in_{filter_id}",
                filter_id=self.id,
            ));
            query_part = query_part.params(format!("value_type_in_{}", self.id), value_type_in.into());
        }

        if let Some(value_type_not_in) = self.value_type_not_in {
            query_part = query_part.where_clause(&format!(
                "val_{filter_id}.value_type NOT IN $value_type_not_in_{filter_id}",
                filter_id=self.id,
            ));
            query_part = query_part.params(format!("value_type_not_in_{}", self.id), value_type_not_in.into());
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
        let query = AttributeFilter::new("n", "name")
            .value("test")
            .value_not("test2")
            .value_in(vec!["test3".to_owned(), "test4".to_owned()])
            .value_not_in(vec!["test5".to_owned(), "test6".to_owned()])
            .into_query_part();

        assert_eq!(
            query,
            QueryPart {
                match_clauses: vec![
                    r#"(n) -[r_n_name:ATTRIBUTE]-> (val_n_name {attribute: "name"})"#.to_owned(),
                ],
                where_clauses: vec![
                    "r_n_name.max_version IS NULL".to_owned(),
                    "val_n_name.value = $value_n_name".to_owned(),
                    "val_n_name.value <> $value_not_n_name".to_owned(),
                    "val_n_name.value IN $value_in_n_name".to_owned(),
                    "val_n_name.value NOT IN $value_not_in_n_name".to_owned(),
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
                ..Default::default()
            },
        )
    }

    #[test]
    fn test_attribute_filter() {
        let filter = AttributeFilter::new("n", "name")
            .value_in(vec!["test".to_string(), "test2".to_string()])
            .value_type("TEXT")
            .space_id("Root");

        let query_part = filter.into_query_part();

        assert_eq!(
            query_part,
            QueryPart {
                match_clauses: vec![
                    r#"(n) -[r_n_name:ATTRIBUTE]-> (val_n_name {attribute: "name"})"#.to_owned(),
                ],
                where_clauses: vec![
                    "r_n_name.space_id = $space_id".to_owned(),
                    "r_n_name.max_version IS NULL".to_owned(),
                    "val_n_name.value IN $value_in_n_name".to_string(),
                    "val_n_name.value_type = $value_type_n_name".to_string(),
                ],
                params: HashMap::from([
                    (
                        "value_in_n_name".to_string(),
                        vec!["test".to_string(), "test2".to_string()].into()
                    ),
                    ("value_type_n_name".to_string(), "TEXT".to_string().into()),
                    ("space_id".to_string(), "Root".to_string().into()),
                ]),
                ..QueryPart::default()
            }
        );

        assert_eq!(
            query_part.query(),
            "MATCH (n) -[r_n_name:ATTRIBUTE]-> (val_n_name {attribute: \"name\"})\nWHERE r_n_name.space_id = $space_id\nAND r_n_name.max_version IS NULL\nAND val_n_name.value IN $value_in_n_name\nAND val_n_name.value_type = $value_type_n_name\n"
        );
    }
}
