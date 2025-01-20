use super::query_part::{IntoQueryPart, QueryPart};

#[derive(Clone, Debug)]
pub struct OrderBy {
    pub(crate) node_var: String,
    pub(crate) property: String,
    pub(crate) order_direction: OrderDirection,
}

impl IntoQueryPart for OrderBy {
    fn into_query_part(self) -> QueryPart {
        let mut query_part = QueryPart::default();

        match self.order_direction {
            OrderDirection::Asc => {
                query_part =
                    query_part.order_by_clause(&format!("{}.`{}`", self.node_var, self.property));
            }
            OrderDirection::Desc => {
                query_part = query_part
                    .order_by_clause(&format!("{}.`{}` DESC", self.node_var, self.property));
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
