use super::query_builder::{MatchQuery, QueryBuilder, Subquery};

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
    pub(crate) fn subquery(&self, node_var: impl Into<String>) -> impl Subquery {
        let node_var = node_var.into();
        let mut query = QueryBuilder::default()
            .subquery(MatchQuery::new(format!(
                r#"({node_var}) -[:ATTRIBUTE]-> ({node_var}_order_by:Attribute {{id: "{}"}})"#,
                self.field_name
            )));

        match self.order_direction {
            OrderDirection::Asc => {
                query = query.subquery(format!("ORDER BY {node_var}_order_by.value"));
            }
            OrderDirection::Desc => {
                query = query.subquery(format!("ORDER BY {node_var}_order_by.value DESC"));
            }
        }

        query
    }
}

#[derive(Clone, Debug, Default)]
pub enum OrderDirection {
    #[default]
    Asc,
    Desc,
}
