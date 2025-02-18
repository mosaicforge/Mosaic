use super::query_part::QueryPart;

#[derive(Clone, Debug)]
pub struct FieldOrderBy {
    pub(crate) field_name: String,
    pub(crate) order_direction: OrderDirection,
}

pub fn asc(field_name: impl Into<String>) -> FieldOrderBy {
    FieldOrderBy {
        field_name: field_name.into(),
        order_direction: OrderDirection::Asc,
    }
}

pub fn desc(field_name: impl Into<String>) -> FieldOrderBy {
    FieldOrderBy {
        field_name: field_name.into(),
        order_direction: OrderDirection::Desc,
    }
}

impl FieldOrderBy {
    pub(crate) fn into_query_part(self, node_var: impl Into<String>) -> QueryPart {
        let node_var = node_var.into();
        let mut query_part = QueryPart::default()
            .match_clause(format!(r#"({node_var}) -[:ATTRIBUTE]-> ({node_var}_order_by:Attribute {{id: "{}"}})"#, self.field_name));

        match self.order_direction {
            OrderDirection::Asc => {
                query_part =
                    query_part.order_by_clause(format!("{node_var}_order_by.value"));
            }
            OrderDirection::Desc => {
                query_part = query_part
                    .order_by_clause(format!("{node_var}_order_by.value DESC"));
            }
        }

        query_part
    }
}

#[derive(Clone, Debug, Default)]
pub enum OrderDirection {
    #[default]
    Asc,
    Desc,
}
