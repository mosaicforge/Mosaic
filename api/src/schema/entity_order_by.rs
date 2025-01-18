use juniper::{GraphQLEnum, GraphQLInputObject};
use sdk::mapping;

#[derive(GraphQLInputObject)]
pub struct EntityOrderBy {
    order_by: String,
    order_direction: Option<OrderDirection>,
}

#[derive(Default, GraphQLEnum)]
pub enum OrderDirection {
    #[default]
    Asc,
    Desc,
}

impl From<OrderDirection> for mapping::OrderDirection {
    fn from(value: OrderDirection) -> Self {
        match value {
            OrderDirection::Asc => Self::Asc,
            OrderDirection::Desc => Self::Desc,
        }
    }
}
