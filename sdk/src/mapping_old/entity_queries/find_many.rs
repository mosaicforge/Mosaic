use std::collections::HashMap;

use crate::mapping::{
    entity_queries::type_filter::TypeFilter,
    query_utils::{
        attributes_filter::AttributeFilter,
        order_by::{FieldOrderBy, OrderDirection},
        query_part::{IntoQueryPart, QueryPart},
        scalar_filter::ScalarFieldFilter,
    },
};

pub struct FindMany {
    node_var: String,

    version: Option<i64>,

    id_filter: ScalarFieldFilter,
    space_filter: ScalarFieldFilter,

    types_filter: TypeFilter,

    attributes_filter: HashMap<String, AttributeFilter>,

    order_by: FieldOrderBy,
}

impl FindMany {
    pub fn new(node_var: &str) -> Self {
        Self {
            node_var: node_var.to_owned(),
            version: None,
            id_filter: ScalarFieldFilter::new(node_var, "id"),
            space_filter: ScalarFieldFilter::new(&format!("r_attr_{node_var}"), "space_id"),
            types_filter: TypeFilter::new(node_var),
            attributes_filter: HashMap::new(),
            order_by: FieldOrderBy {
                node_var: node_var.to_owned(),
                field_name: "id".to_owned(),
                order_direction: Default::default(),
            },
        }
    }

    pub fn version(mut self, version: i64) -> Self {
        self.version = Some(version);

        for attribute_filter in self.attributes_filter.values_mut() {
            attribute_filter.version_mut(version);
        }

        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id_filter = self.id_filter.value(id);
        self
    }

    pub fn id_mut(&mut self, id: &str) {
        self.id_filter.value_mut(id);
    }

    pub fn id_not(mut self, id: &str) -> Self {
        self.id_filter = self.id_filter.value_not(id);
        self
    }

    pub fn id_not_mut(&mut self, id: &str) {
        self.id_filter.value_not_mut(id);
    }

    pub fn id_in(mut self, ids: Vec<String>) -> Self {
        self.id_filter = self.id_filter.value_in(ids);
        self
    }

    pub fn id_in_mut(&mut self, ids: Vec<String>) {
        self.id_filter.value_in_mut(ids);
    }

    pub fn id_not_in(mut self, ids: Vec<String>) -> Self {
        self.id_filter = self.id_filter.value_not_in(ids);
        self
    }

    pub fn id_not_in_mut(&mut self, ids: Vec<String>) {
        self.id_filter.value_not_in_mut(ids);
    }

    pub fn space_id(mut self, space_id: &str) -> Self {
        self.space_filter = self.space_filter.value(space_id);
        self
    }

    pub fn types(mut self, types: Vec<String>) -> Self {
        self.types_filter = self.types_filter.types(types);
        self
    }

    pub fn types_mut(&mut self, types: Vec<String>) {
        self.types_filter.types_mut(types);
    }

    pub fn types_not(mut self, types: Vec<String>) -> Self {
        self.types_filter = self.types_filter.types_not(types);
        self
    }

    pub fn types_not_mut(&mut self, types: Vec<String>) {
        self.types_filter.types_not_mut(types);
    }

    pub fn types_contains(mut self, types: Vec<String>) -> Self {
        self.types_filter = self.types_filter.types_contains(types);
        self
    }

    pub fn types_contains_mut(&mut self, types: Vec<String>) {
        self.types_filter.types_contains_mut(types);
    }

    pub fn types_not_contains(mut self, types: Vec<String>) -> Self {
        self.types_filter = self.types_filter.types_not_contains(types);
        self
    }

    pub fn types_not_contains_mut(&mut self, types: Vec<String>) {
        self.types_filter.types_not_contains_mut(types);
    }

    pub fn attribute(mut self, attribute: &str, value: &str) -> Self {
        self.attributes_filter
            .entry(attribute.to_owned())
            .or_insert_with(|| AttributeFilter::new(&self.node_var, attribute).space_id_opt(self.space_filter.value.clone()))
            .value_mut(value);
        self
    }

    pub fn attribute_not(mut self, attribute: &str, value: &str) -> Self {
        self.attributes_filter
            .entry(attribute.to_owned())
            .or_insert_with(|| AttributeFilter::new(&self.node_var, attribute).space_id_opt(self.space_filter.value.clone()))
            .value_not_mut(value);
        self
    }

    pub fn attribute_in(mut self, attribute: &str, values: Vec<String>) -> Self {
        self.attributes_filter
            .entry(attribute.to_owned())
            .or_insert_with(|| AttributeFilter::new(&self.node_var, attribute).space_id_opt(self.space_filter.value.clone()))
            .value_in_mut(values);
        self
    }

    pub fn attribute_not_in(mut self, attribute: &str, values: Vec<String>) -> Self {
        self.attributes_filter
            .entry(attribute.to_owned())
            .or_insert_with(|| AttributeFilter::new(&self.node_var, attribute).space_id_opt(self.space_filter.value.clone()))
            .value_not_in_mut(values);
        self
    }

    pub fn attribute_value_type(mut self, attribute: &str, value_type: &str) -> Self {
        self.attributes_filter
            .entry(attribute.to_owned())
            .or_insert_with(|| AttributeFilter::new(&self.node_var, attribute).space_id_opt(self.space_filter.value.clone()))
            .value_type_mut(value_type);
        self
    }

    pub fn attribute_value_type_not(mut self, attribute: &str, value_type: &str) -> Self {
        self.attributes_filter
            .entry(attribute.to_owned())
            .or_insert_with(|| AttributeFilter::new(&self.node_var, attribute).space_id_opt(self.space_filter.value.clone()))
            .value_type_not_mut(value_type);
        self
    }

    pub fn attribute_value_type_in(mut self, attribute: &str, value_types: Vec<String>) -> Self {
        self.attributes_filter
            .entry(attribute.to_owned())
            .or_insert_with(|| AttributeFilter::new(&self.node_var, attribute).space_id_opt(self.space_filter.value.clone()))
            .value_type_in_mut(value_types);
        self
    }

    pub fn attribute_value_type_not_in(
        mut self,
        attribute: &str,
        value_types: Vec<String>,
    ) -> Self {
        self.attributes_filter
            .entry(attribute.to_owned())
            .or_insert_with(|| AttributeFilter::new(&self.node_var, attribute).space_id_opt(self.space_filter.value.clone()))
            .value_type_not_in_mut(value_types);
        self
    }

    pub fn order_by(mut self, field_name: &str) -> Self {
        self.order_by.field_name = field_name.to_owned();
        self
    }

    pub fn order_by_mut(&mut self, field_name: &str) {
        self.order_by.field_name = field_name.to_owned();
    }

    pub fn order_direction(mut self, order_direction: OrderDirection) -> Self {
        self.order_by.order_direction = order_direction;
        self
    }

    pub fn order_direction_mut(&mut self, order_direction: OrderDirection) {
        self.order_by.order_direction = order_direction;
    }
}

impl IntoQueryPart for FindMany {
    fn into_query_part(self) -> QueryPart {
        let mut query_part = QueryPart::default();

        query_part = query_part.match_clause(
            &format!(
                r#"({node_var}) -[r_attr_{node_var}:ATTRIBUTE]-> (attr_{node_var})"#,
                node_var = self.node_var
            )
        );

        if let Some(version) = self.version {
            query_part = query_part.where_clause(&format!(
                "r_attr_{node_var}.min_version =< $version AND (r_attr_{node_var}.max_version > $version OR r_attr_{node_var}.max_version IS NULL)",
                node_var = self.node_var,
            ));

            query_part.params.insert("version".to_owned(), version.into());
        } else {
            query_part = query_part.where_clause(&format!("r_attr_{node_var}.max_version IS NULL", node_var = self.node_var));
        }

        query_part.merge_mut(self.types_filter.into_query_part());
        query_part.merge_mut(self.id_filter.into_query_part());
        query_part.merge_mut(self.space_filter.into_query_part());
        query_part.merge_mut(self.order_by.into_query_part());

        for attribute_filter in self.attributes_filter.into_values() {
            query_part.merge_mut(attribute_filter.into_query_part());
        }

        if query_part.match_clauses.is_empty() {
            query_part = query_part.match_clause(&format!("({})", self.node_var));
        }

        query_part
            .with_clause(&self.node_var)
            .with_clause(&format!("collect(attr_{node_var}{{.*}}) AS attributes", node_var = self.node_var))
            .return_clause(&format!("{node_var}{{.*, attributes: attributes}}", node_var = self.node_var))

        // query_part
        //     .return_clause(&self.node_var)
        //     .return_clause(&format!("r_attr_{}", self.node_var))
        //     .return_clause(&format!("attr_{}", self.node_var))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    use crate::system_ids;

    use super::*;

    #[test]
    fn test_default() {
        let query_part = FindMany::new("n").into_query_part();

        assert_eq!(
            query_part,
            QueryPart {
                match_clauses: vec![
                    "(n) -[r_attr_n:ATTRIBUTE]-> (attr_n)".to_owned(),
                ],
                where_clauses: vec![
                    "r_attr_n.max_version IS NULL".to_owned(),
                ],
                order_by_clauses: vec!["n.`id`".to_owned(),],
                with_clauses: vec![
                    "n".to_owned(),
                    "collect(attr_n{.*}) AS attributes".to_owned(),
                ],
                return_clauses: vec!["n{.*, attributes: attributes}".to_owned()],
                ..Default::default()
            }
        );

        assert_eq!(
            query_part.query(),
            r#"MATCH (n) -[r_attr_n:ATTRIBUTE]-> (attr_n)
WHERE r_attr_n.max_version IS NULL
WITH n, collect(attr_n{.*}) AS attributes
RETURN n{.*, attributes: attributes}
ORDER BY n.`id`"#,
        );
    }

    #[test]
    fn test_find_many() {
        let query_part = FindMany::new("n")
            .id("abc")
            .space_id("Root")
            .types(vec!["Type".to_owned()])
            .attribute("name", "test")
            .attribute_value_type("name", "TEXT")
            .into_query_part();

        assert_eq!(
            query_part,
            QueryPart {
                match_clauses: vec![
                    "(n) -[r_attr_n:ATTRIBUTE]-> (attr_n)".to_owned(),
                    format!(
                        "(n) <-[:`{FROM_ENTITY}`]- (n_r) -[:`{TO_ENTITY}`]-> (n_t)",
                        FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
                        TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
                    ),
                    format!(
                        "(n_r) -[:`{RELATION_TYPE}`]-> ({{id: \"{TYPES}\"}})",
                        RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
                        TYPES = system_ids::TYPES_ATTRIBUTE,
                    ),
                    r#"(n) -[r_n_name:ATTRIBUTE]-> (val_n_name {attribute: "name"})"#.to_owned(),
                ],
                where_clauses: vec![
                    "r_attr_n.max_version IS NULL".to_owned(),
                    "n_t.`id` = $value_n_t_id".to_owned(),
                    "n.`id` = $value_n_id".to_owned(),
                    "r_attr_n.`space_id` = $space_id".to_owned(),
                    "r_n_name.space_id = $space_id".to_owned(),
                    "r_n_name.max_version IS NULL".to_owned(),
                    "val_n_name.value = $value_n_name".to_owned(),
                    "val_n_name.value_type = $value_type_n_name".to_owned(),
                ],
                order_by_clauses: vec!["n.`id`".to_owned(),],
                with_clauses: vec![
                    "n".to_owned(),
                    "collect(attr_n{.*}) AS attributes".to_owned(),
                ],
                return_clauses: vec!["n{.*, attributes: attributes}".to_owned()],
                params: HashMap::from([
                    ("value_n_id".to_owned(), "abc".into()),
                    ("value_n_t_id".to_owned(), vec!["Type".to_string()].into()),
                    ("value_n_name".to_owned(), "test".into()),
                    ("value_type_n_name".to_owned(), "TEXT".into()),
                    ("space_id".to_owned(), "Root".into()),
                ]),
                ..Default::default()
            }
        );

        assert_eq!(
            query_part.query(),
            r#"MATCH (n) -[r_attr_n:ATTRIBUTE]-> (attr_n)
MATCH (n) <-[:`RERshk4JoYoMC17r1qAo9J`]- (n_r) -[:`Qx8dASiTNsxxP3rJbd4Lzd`]-> (n_t)
MATCH (n_r) -[:`3WxYoAVreE4qFhkDUs5J3q`]-> ({id: "Jfmby78N4BCseZinBmdVov"})
MATCH (n) -[r_n_name:ATTRIBUTE]-> (val_n_name {attribute: "name"})
WHERE r_attr_n.max_version IS NULL
AND n_t.`id` = $value_n_t_id
AND n.`id` = $value_n_id
AND r_attr_n.`space_id` = $space_id
AND r_n_name.space_id = $space_id
AND r_n_name.max_version IS NULL
AND val_n_name.value = $value_n_name
AND val_n_name.value_type = $value_type_n_name
WITH n, collect(attr_n{.*}) AS attributes
RETURN n{.*, attributes: attributes}
ORDER BY n.`id`"#,
        );
    }

    #[test]
    fn test_find_many_no_type() {
        let query_part = FindMany::new("n")
            .id("abc")
            .attribute("name", "test")
            .attribute_value_type("name", "TEXT")
            .into_query_part();

        assert_eq!(
            query_part,
            QueryPart {
                match_clauses: vec![
                    "(n) -[r_attr_n:ATTRIBUTE]-> (attr_n)".to_owned(),
                    r#"(n) -[r_n_name:ATTRIBUTE]-> (val_n_name {attribute: "name"})"#.to_owned(),
                ],
                where_clauses: vec![
                    "r_attr_n.max_version IS NULL".to_owned(),
                    "n.`id` = $value_n_id".to_owned(),
                    "r_n_name.max_version IS NULL".to_owned(),
                    "val_n_name.value = $value_n_name".to_owned(),
                    "val_n_name.value_type = $value_type_n_name".to_owned(),
                ],
                order_by_clauses: vec!["n.`id`".to_owned(),],
                with_clauses: vec![
                    "n".to_owned(),
                    "collect(attr_n{.*}) AS attributes".to_owned(),
                ],
                return_clauses: vec!["n{.*, attributes: attributes}".to_owned()],
                params: HashMap::from([
                    ("value_n_id".to_owned(), "abc".into()),
                    ("value_n_name".to_owned(), "test".into()),
                    ("value_type_n_name".to_owned(), "TEXT".into()),
                ]),
                ..Default::default()
            }
        );

        assert_eq!(
            query_part.query(),
            r#"MATCH (n) -[r_attr_n:ATTRIBUTE]-> (attr_n)
MATCH (n) -[r_n_name:ATTRIBUTE]-> (val_n_name {attribute: "name"})
WHERE r_attr_n.max_version IS NULL
AND n.`id` = $value_n_id
AND r_n_name.max_version IS NULL
AND val_n_name.value = $value_n_name
AND val_n_name.value_type = $value_type_n_name
WITH n, collect(attr_n{.*}) AS attributes
RETURN n{.*, attributes: attributes}
ORDER BY n.`id`"#,
        );
    }

    #[test]
    pub fn test_find_many_version() {
        let query_part = FindMany::new("n")
            .id("abc")
            .version(2)
            .attribute("name", "test")
            .attribute_value_type("name", "TEXT")
            .into_query_part();

        assert_eq!(
            query_part,
            QueryPart {
                match_clauses: vec![
                    "(n) -[r_attr_n:ATTRIBUTE]-> (attr_n)".to_owned(),
                    r#"(n) -[r_n_name:ATTRIBUTE]-> (val_n_name {attribute: "name"})"#.to_owned(),
                ],
                where_clauses: vec![
                    "r_attr_n.min_version =< $version AND (r_attr_n.max_version > $version OR r_attr_n.max_version IS NULL)".to_owned(),
                    "n.`id` = $value_n_id".to_owned(),
                    "r_n_name.max_version IS NULL".to_owned(),
                    "val_n_name.value = $value_n_name".to_owned(),
                    "val_n_name.value_type = $value_type_n_name".to_owned(),
                ],
                with_clauses: vec![
                    "n".to_owned(),
                    "collect(attr_n{.*}) AS attributes".to_owned(),
                ],
                order_by_clauses: vec!["n.`id`".to_owned(),],
                return_clauses: vec!["n{.*, attributes: attributes}".to_owned()],
                params: HashMap::from([
                    ("version".to_owned(), 2.into()),
                    ("value_n_id".to_owned(), "abc".into()),
                    ("value_n_name".to_owned(), "test".into()),
                    ("value_type_n_name".to_owned(), "TEXT".into()),
                ]),
                ..Default::default()
            }
        );

        assert_eq!(
            query_part.query(),
            r#"MATCH (n) -[r_attr_n:ATTRIBUTE]-> (attr_n)
MATCH (n) -[r_n_name:ATTRIBUTE]-> (val_n_name {attribute: "name"})
WHERE r_attr_n.min_version =< $version AND (r_attr_n.max_version > $version OR r_attr_n.max_version IS NULL)
AND n.`id` = $value_n_id
AND r_n_name.max_version IS NULL
AND val_n_name.value = $value_n_name
AND val_n_name.value_type = $value_type_n_name
WITH n, collect(attr_n{.*}) AS attributes
RETURN n{.*, attributes: attributes}
ORDER BY n.`id`"#,
        );
    }
}
