use std::collections::HashMap;

use crate::{
    mapping::{
        entity_queries,
        query_utils::{
            property_filter::PropertyFilter,
            order_by::OrderBy,
            query_part::{IntoQueryPart, QueryPart},
            scalar_filter::ScalarPropertyFilter,
        },
        OrderDirection,
    },
    system_ids,
};

pub struct FindMany {
    node_var: String,

    id_filter: ScalarPropertyFilter,
    space_filter: ScalarPropertyFilter,

    relation_type_filter: ScalarPropertyFilter,

    to_filter: entity_queries::FindMany,
    from_filter: entity_queries::FindMany,

    properties_filter: HashMap<String, PropertyFilter>,

    order_by: OrderBy,
}

impl FindMany {
    pub fn new(node_var: &str) -> Self {
        Self {
            node_var: node_var.to_owned(),
            id_filter: ScalarPropertyFilter::new(node_var, "id"),
            space_filter: ScalarPropertyFilter::new(node_var, "space_id"),
            relation_type_filter: ScalarPropertyFilter::new("rt", "id"),
            to_filter: entity_queries::FindMany::new("to"),
            from_filter: entity_queries::FindMany::new("from"),
            properties_filter: HashMap::new(),
            order_by: OrderBy {
                node_var: node_var.to_owned(),
                property: system_ids::RELATION_INDEX.to_owned(),
                order_direction: Default::default(),
            },
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id_filter = self.id_filter.value(id);
        self
    }

    pub fn id_not(mut self, id: &str) -> Self {
        self.id_filter = self.id_filter.value_not(id);
        self
    }

    pub fn id_in(mut self, ids: Vec<String>) -> Self {
        self.id_filter = self.id_filter.value_in(ids);
        self
    }

    pub fn id_not_in(mut self, ids: Vec<String>) -> Self {
        self.id_filter = self.id_filter.value_not_in(ids);
        self
    }

    pub fn space_id(mut self, space_id: &str) -> Self {
        self.space_filter = self.space_filter.value(space_id);
        self
    }

    pub fn relation_type(mut self, relation_type: &str) -> Self {
        self.relation_type_filter = self.relation_type_filter.value(relation_type);
        self
    }

    pub fn relation_type_not(mut self, relation_type: &str) -> Self {
        self.relation_type_filter = self.relation_type_filter.value_not(relation_type);
        self
    }

    pub fn relation_type_in(mut self, relation_types: Vec<String>) -> Self {
        self.relation_type_filter = self.relation_type_filter.value_in(relation_types);
        self
    }

    pub fn relation_type_not_in(mut self, relation_types: Vec<String>) -> Self {
        self.relation_type_filter = self.relation_type_filter.value_not_in(relation_types);
        self
    }

    pub fn to(
        mut self,
        f: impl FnOnce(entity_queries::FindMany) -> entity_queries::FindMany,
    ) -> Self {
        self.to_filter = f(self.to_filter);
        self
    }

    pub fn from(
        mut self,
        f: impl FnOnce(entity_queries::FindMany) -> entity_queries::FindMany,
    ) -> Self {
        self.from_filter = f(self.from_filter);
        self
    }

    pub fn attribute(mut self, property: &str, value: &str) -> Self {
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

    pub fn order_direction(mut self, order_direction: OrderDirection) -> Self {
        self.order_by.order_direction = order_direction;
        self
    }
}

impl IntoQueryPart for FindMany {
    fn into_query_part(self) -> QueryPart {
        let mut base_query = QueryPart::default()
            .match_clause(&format!(
                "(from) <-[:`{FROM_ENTITY}`]- ({node_var}) -[:`{TO_ENTITY}`]-> (to)",
                FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
                TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
                node_var = self.order_by.node_var,
            ))
            .match_clause(&format!(
                "({node_var}) -[:`{RELATION_TYPE}`]-> (rt)",
                RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
                node_var = self.order_by.node_var,
            ))
            .return_clause("from")
            .return_clause("to")
            .return_clause("r")
            .return_clause("rt");

        base_query.merge_mut(self.id_filter.into_query_part());
        base_query.merge_mut(self.space_filter.into_query_part());
        base_query.merge_mut(self.relation_type_filter.into_query_part());
        base_query.merge_mut(self.to_filter.into_query_part());
        base_query.merge_mut(self.from_filter.into_query_part());

        for attribute_filter in self.properties_filter.into_values() {
            base_query.merge_mut(attribute_filter.into_query_part());
        }

        base_query.merge_mut(self.order_by.into_query_part());

        tracing::info!("relation_queries::FindMany query:\n{}", base_query.query());

        base_query
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_basic() {
        let find_many = FindMany::new("r")
            .id("abc")
            .relation_type("test_relation_type")
            .attribute("attr", "value")
            .property_not("attr", "value_not")
            .property_in("attr", vec!["value_in".to_string()])
            .property_not_in("attr", vec!["value_not_in".to_string()])
            .property_value_type("attr", "value_type")
            .property_value_type_not("attr", "value_type_not")
            .property_value_type_in("attr", vec!["value_type_in".to_string()])
            .property_value_type_not_in("attr", vec!["value_type_not_in".to_string()]);

        let query_part = find_many.into_query_part();

        assert_eq!(
            query_part,
            QueryPart {
                match_clauses: vec![
                    "(from) <-[:`RERshk4JoYoMC17r1qAo9J`]- (r) -[:`Qx8dASiTNsxxP3rJbd4Lzd`]-> (to)"
                        .to_owned(),
                    "(r) -[:`3WxYoAVreE4qFhkDUs5J3q`]-> (rt)".to_owned(),
                ],
                where_clauses: vec![
                    "r.`id` = $value_r_id".to_owned(),
                    "rt.`id` = $value_rt_id".to_owned(),
                    "r.`attr` = $value_r_attr".to_owned(),
                    "r.`attr` <> $value_not_r_attr".to_owned(),
                    "r.`attr` IN $value_in_r_attr".to_owned(),
                    "r.`attr` NOT IN $value_not_in_r_attr".to_owned(),
                    "r.`attr.type` = $value_r_attr_type".to_owned(),
                    "r.`attr.type` <> $value_not_r_attr_type".to_owned(),
                    "r.`attr.type` IN $value_in_r_attr_type".to_owned(),
                    "r.`attr.type` NOT IN $value_not_in_r_attr_type".to_owned(),
                ],
                order_by_clauses: vec![format!("r.`{}`", system_ids::RELATION_INDEX)],
                return_clauses: vec![
                    "from".to_owned(),
                    "to".to_owned(),
                    "r".to_owned(),
                    "rt".to_owned()
                ],
                params: HashMap::from([
                    ("value_r_id".to_owned(), "abc".into()),
                    ("value_rt_id".to_owned(), "test_relation_type".into()),
                    ("value_r_attr".to_owned(), "value".into()),
                    ("value_not_r_attr".to_owned(), "value_not".into()),
                    (
                        "value_in_r_attr".to_owned(),
                        vec!["value_in".to_string()].into()
                    ),
                    (
                        "value_not_in_r_attr".to_owned(),
                        vec!["value_not_in".to_string()].into()
                    ),
                    ("value_r_attr_type".to_owned(), "value_type".into()),
                    ("value_not_r_attr_type".to_owned(), "value_type_not".into()),
                    (
                        "value_in_r_attr_type".to_owned(),
                        vec!["value_type_in".to_string()].into()
                    ),
                    (
                        "value_not_in_r_attr_type".to_owned(),
                        vec!["value_type_not_in".to_string()].into()
                    ),
                ]),
            }
        );

        assert_eq!(
            query_part.query(),
            r#"MATCH (from) <-[:`RERshk4JoYoMC17r1qAo9J`]- (r) -[:`Qx8dASiTNsxxP3rJbd4Lzd`]-> (to)
MATCH (r) -[:`3WxYoAVreE4qFhkDUs5J3q`]-> (rt)
WHERE r.`id` = $value_r_id
AND rt.`id` = $value_rt_id
AND r.`attr` = $value_r_attr
AND r.`attr` <> $value_not_r_attr
AND r.`attr` IN $value_in_r_attr
AND r.`attr` NOT IN $value_not_in_r_attr
AND r.`attr.type` = $value_r_attr_type
AND r.`attr.type` <> $value_not_r_attr_type
AND r.`attr.type` IN $value_in_r_attr_type
AND r.`attr.type` NOT IN $value_not_in_r_attr_type
RETURN from, to, r, rt
ORDER BY r.`WNopXUYxsSsE51gkJGWghe`"#,
        )
    }

    fn test_from_filter() {}

    fn test_to_filter() {}

    fn test_to_from_filter() {}
}
