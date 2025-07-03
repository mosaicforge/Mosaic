use crate::error::DatabaseError;
use futures::Stream;

pub mod attributes_filter;
pub mod order_by;
pub mod prop_filter;
pub mod query_builder;
pub mod query_part;
pub mod relation_direction;
pub mod version_filter;

pub use attributes_filter::AttributeFilter;
pub use order_by::{FieldOrderBy, OrderDirection};
pub use prop_filter::PropFilter;
pub use query_part::QueryPart;
pub use relation_direction::RelationDirection;
pub use version_filter::VersionFilter;

pub trait Query<T>: Sized {
    fn send(self) -> impl std::future::Future<Output = Result<T, DatabaseError>>;
}

pub trait QueryStream<T>: Sized {
    fn send(
        self,
    ) -> impl std::future::Future<
        Output = Result<impl Stream<Item = Result<T, DatabaseError>>, DatabaseError>,
    >;
}
