use crate::{
    mapping::query_utils::{
        list_filter::ListFieldFilter,
        query_part::{IntoQueryPart, QueryPart},
    },
    system_ids,
};

pub struct TypeFilter {
    node_var: String,
    filter: ListFieldFilter,
}

impl TypeFilter {
    pub fn new(node_var: &str) -> Self {
        Self {
            node_var: node_var.to_string(),
            filter: ListFieldFilter::new(&format!("{}_t", node_var), "id"),
        }
    }

    pub fn with_id(node_var: &str, id: &str) -> Self {
        Self {
            node_var: node_var.to_string(),
            filter: ListFieldFilter::with_id(&format!("{}_t", node_var), "id", id),
        }
    }

    pub fn types(mut self, types: Vec<String>) -> Self {
        self.filter = self.filter.value(types);
        self
    }

    pub fn types_mut(&mut self, types: Vec<String>) {
        self.filter.value_mut(types);
    }

    pub fn types_not(mut self, types: Vec<String>) -> Self {
        self.filter = self.filter.value_not(types);
        self
    }

    pub fn types_not_mut(&mut self, types: Vec<String>) {
        self.filter.value_not_mut(types);
    }

    pub fn types_contains(mut self, types: Vec<String>) -> Self {
        self.filter = self.filter.value_contains(types);
        self
    }

    pub fn types_contains_mut(&mut self, types: Vec<String>) {
        self.filter.value_contains_mut(types);
    }

    pub fn types_not_contains(mut self, types: Vec<String>) -> Self {
        self.filter = self.filter.value_not_contains(types);
        self
    }

    pub fn types_not_contains_mut(&mut self, types: Vec<String>) {
        self.filter.value_not_contains_mut(types);
    }
}

impl IntoQueryPart for TypeFilter {
    fn into_query_part(self) -> QueryPart {
        let query_part = self.filter.into_query_part();

        if !query_part.is_empty() {
            query_part
                .match_clause(&format!(
                    "({node_var}) <-[:`{FROM_ENTITY}`]- ({node_var}_r) -[:`{TO_ENTITY}`]-> ({node_var}_t)",
                    FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
                    TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
                    node_var = self.node_var,
                ))
                .match_clause(&format!(
                    "({node_var}_r) -[:`{RELATION_TYPE}`]-> ({{id: \"{TYPES}\"}})",
                    RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
                    TYPES = system_ids::TYPES_ATTRIBUTE,
                    node_var = self.node_var,
                ))
        } else {
            query_part
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_type_filter() {
        let query_part = TypeFilter::new("n")
            .types(vec!["test1".to_string(), "test2".to_string()])
            .types_not(vec!["test3".to_string(), "test4".to_string()])
            .types_contains(vec!["test5".to_string(), "test6".to_string()])
            .types_not_contains(vec!["test7".to_string(), "test8".to_string()])
            .into_query_part();

        assert_eq!(
            query_part,
            QueryPart {
                match_clauses: vec![
                    format!(
                        "(n) <-[:`{FROM_ENTITY}`]- (n_r) -[:`{TO_ENTITY}`]-> (n_t)",
                        FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
                        TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
                    ),
                    format!(
                        "(n_r) -[:`{RELATION_TYPE}`]-> ({{id: \"{TYPES}\"}})",
                        RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
                        TYPES = system_ids::TYPES_ATTRIBUTE,
                    )
                ],
                where_clauses: vec![
                    "n_t.`id` = $value_n_t_id".to_owned(),
                    "n_t.`id` <> $value_not_n_t_id".to_owned(),
                    "ALL(x IN $value_contains_n_t_id WHERE x IN n_t.`id`)".to_owned(),
                    "NOT ANY(x IN $value_not_contains_n_t_id WHERE x IN n_t.`id`)".to_owned(),
                ],
                params: HashMap::from([
                    (
                        "value_n_t_id".to_owned(),
                        vec!["test1".to_string(), "test2".to_string()].into()
                    ),
                    (
                        "value_not_n_t_id".to_owned(),
                        vec!["test3".to_string(), "test4".to_string()].into()
                    ),
                    (
                        "value_contains_n_t_id".to_owned(),
                        vec!["test5".to_string(), "test6".to_string()].into()
                    ),
                    (
                        "value_not_contains_n_t_id".to_owned(),
                        vec!["test7".to_string(), "test8".to_string()].into()
                    ),
                ]),
                ..QueryPart::default()
            },
        );

        assert_eq!(
            query_part.query(),
            r#"MATCH (n) <-[:`RERshk4JoYoMC17r1qAo9J`]- (n_r) -[:`Qx8dASiTNsxxP3rJbd4Lzd`]-> (n_t)
MATCH (n_r) -[:`3WxYoAVreE4qFhkDUs5J3q`]-> ({id: "Jfmby78N4BCseZinBmdVov"})
WHERE n_t.`id` = $value_n_t_id
AND n_t.`id` <> $value_not_n_t_id
AND ALL(x IN $value_contains_n_t_id WHERE x IN n_t.`id`)
AND NOT ANY(x IN $value_not_contains_n_t_id WHERE x IN n_t.`id`)
"#
        )
    }
}
