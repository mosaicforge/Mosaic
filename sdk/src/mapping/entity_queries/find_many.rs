use std::collections::HashMap;

use crate::mapping::{
    entity_queries::type_filter::TypeFilter,
    query_utils::{
        property_filter::PropertyFilter,
        order_by::{OrderBy, OrderDirection},
        query_part::{IntoQueryPart, QueryPart},
        scalar_filter::ScalarPropertyFilter,
    },
};

pub struct FindMany {
    node_var: String,

    id_filter: ScalarPropertyFilter,
    space_filter: ScalarPropertyFilter,

    types_filter: TypeFilter,

    properties_filter: HashMap<String, PropertyFilter>,

    order_by: OrderBy,
}

impl FindMany {
    pub fn new(node_var: &str) -> Self {
        Self {
            node_var: node_var.to_owned(),
            id_filter: ScalarPropertyFilter::new(node_var, "id"),
            space_filter: ScalarPropertyFilter::new(node_var, "space_id"),
            types_filter: TypeFilter::new(node_var),
            properties_filter: HashMap::new(),
            order_by: OrderBy {
                node_var: node_var.to_owned(),
                property: "id".to_owned(),
                order_direction: Default::default(),
            },
        }
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

    pub fn property(mut self, property: &str, value: &str) -> Self {
        self.properties_filter
            .entry(property.to_owned())
            .or_insert_with(|| PropertyFilter::new(&self.node_var, property))
            .value_mut(value);
        self
    }

    pub fn property_not(mut self, property: &str, value: &str) -> Self {
        self.properties_filter
            .entry(property.to_owned())
            .or_insert_with(|| PropertyFilter::new(&self.node_var, property))
            .value_not_mut(value);
        self
    }

    pub fn property_in(mut self, property: &str, values: Vec<String>) -> Self {
        self.properties_filter
            .entry(property.to_owned())
            .or_insert_with(|| PropertyFilter::new(&self.node_var, property))
            .value_in_mut(values);
        self
    }

    pub fn property_not_in(mut self, property: &str, values: Vec<String>) -> Self {
        self.properties_filter
            .entry(property.to_owned())
            .or_insert_with(|| PropertyFilter::new(&self.node_var, property))
            .value_not_in_mut(values);
        self
    }

    pub fn property_value_type(mut self, property: &str, value_type: &str) -> Self {
        self.properties_filter
            .entry(property.to_owned())
            .or_insert_with(|| PropertyFilter::new(&self.node_var, property))
            .value_type_mut(value_type);
        self
    }

    pub fn property_value_type_not(mut self, property: &str, value_type: &str) -> Self {
        self.properties_filter
            .entry(property.to_owned())
            .or_insert_with(|| PropertyFilter::new(&self.node_var, property))
            .value_type_not_mut(value_type);
        self
    }

    pub fn property_value_type_in(mut self, property: &str, value_types: Vec<String>) -> Self {
        self.properties_filter
            .entry(property.to_owned())
            .or_insert_with(|| PropertyFilter::new(&self.node_var, property))
            .value_type_in_mut(value_types);
        self
    }

    pub fn property_value_type_not_in(
        mut self,
        property: &str,
        value_types: Vec<String>,
    ) -> Self {
        self.properties_filter
            .entry(property.to_owned())
            .or_insert_with(|| PropertyFilter::new(&self.node_var, property))
            .value_type_not_in_mut(value_types);
        self
    }

    pub fn order_by(mut self, property: &str) -> Self {
        self.order_by.property = property.to_owned();
        self
    }

    pub fn order_by_mut(&mut self, property: &str) {
        self.order_by.property = property.to_owned();
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
        // If no types filter is set, we set the match clause
        let mut base_query = {
            let type_filter = self.types_filter.into_query_part();
            if type_filter.is_empty() {
                QueryPart::default()
                    .match_clause(&format!("({})", self.node_var))
                    .return_clause(&self.node_var)
            } else {
                type_filter.return_clause(&self.node_var)
            }
        };

        base_query.merge_mut(self.id_filter.into_query_part());
        base_query.merge_mut(self.space_filter.into_query_part());
        base_query.merge_mut(self.order_by.into_query_part());

        for attribute_filter in self.properties_filter.into_values() {
            base_query.merge_mut(attribute_filter.into_query_part());
        }

        tracing::info!("entity_queries::FindMany query:\n{}", base_query.query());

        base_query
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    use crate::system_ids;

    use super::*;

    #[test]
    fn test_find_many() {
        let query_part = FindMany::new("n")
            .id("abc")
            .types(vec!["Type".to_owned()])
            .property("name", "test")
            .property_value_type("name", "TEXT")
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
                    "n.`id` = $value_n_id".to_owned(),
                    "n.`name` = $value_n_name".to_owned(),
                    "n.`name.type` = $value_n_name_type".to_owned(),
                ],
                order_by_clauses: vec!["n.`id`".to_owned(),],
                return_clauses: vec!["n".to_owned(),],
                params: HashMap::from([
                    ("value_n_id".to_owned(), "abc".into()),
                    ("value_n_t_id".to_owned(), vec!["Type".to_string()].into()),
                    ("value_n_name".to_owned(), "test".into()),
                    ("value_n_name_type".to_owned(), "TEXT".into()),
                ]),
            }
        );

        assert_eq!(
            query_part.query(),
            r#"MATCH (n) <-[:`RERshk4JoYoMC17r1qAo9J`]- (n_r) -[:`Qx8dASiTNsxxP3rJbd4Lzd`]-> (n_t)
MATCH (n_r) -[:`3WxYoAVreE4qFhkDUs5J3q`]-> ({id: "Jfmby78N4BCseZinBmdVov"})
WHERE n_t.`id` = $value_n_t_id
AND n.`id` = $value_n_id
AND n.`name` = $value_n_name
AND n.`name.type` = $value_n_name_type
RETURN n
ORDER BY n.`id`"#,
        );
    }

    #[test]
    fn test_find_many_no_type() {
        let query_part = FindMany::new("n")
            .id("abc")
            .property("name", "test")
            .property_value_type("name", "TEXT")
            .into_query_part();

        assert_eq!(
            query_part,
            QueryPart {
                match_clauses: vec!["(n)".to_owned()],
                where_clauses: vec![
                    "n.`id` = $value_n_id".to_owned(),
                    "n.`name` = $value_n_name".to_owned(),
                    "n.`name.type` = $value_n_name_type".to_owned(),
                ],
                order_by_clauses: vec!["n.`id`".to_owned(),],
                return_clauses: vec!["n".to_owned(),],
                params: HashMap::from([
                    ("value_n_id".to_owned(), "abc".into()),
                    ("value_n_name".to_owned(), "test".into()),
                    ("value_n_name_type".to_owned(), "TEXT".into()),
                ]),
            }
        );

        assert_eq!(
            query_part.query(),
            r#"MATCH (n)
WHERE n.`id` = $value_n_id
AND n.`name` = $value_n_name
AND n.`name.type` = $value_n_name_type
RETURN n
ORDER BY n.`id`"#,
        );
    }
}
