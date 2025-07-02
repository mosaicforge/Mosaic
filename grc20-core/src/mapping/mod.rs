pub mod aggregation;
pub mod attribute_node;
pub mod attributes;
pub mod entity;
pub mod entity_version;
pub mod error;
pub mod pluralism;
pub mod query_utils;
pub mod relation;
pub mod triple;
pub mod value;

pub use aggregation::AggregationDirection;
pub use attribute_node::AttributeNode;
pub use attributes::{Attributes, FromAttributes, IntoAttributes};
pub use entity::{Entity, EntityFilter, EntityNode, EntityNodeRef, EntityRelationFilter};
pub use entity_version::EntityVersion;
pub use error::TriplesConversionError;
pub use pluralism::Pluralism;
pub use query_utils::{
    order_by, prop_filter,
    query_builder::{MatchQuery, QueryBuilder, Subquery, WhereClause},
    AttributeFilter, PropFilter, Query, QueryStream,
};
pub use relation::{Relation, RelationEdge};
pub use triple::Triple;
pub use value::{Options, Value, ValueType};

use crate::{error::DatabaseError, indexer_ids};

pub fn new_version_index(block_number: u64, idx: usize) -> String {
    format!("{block_number:016}:{idx:04}")
}

pub async fn get_version_index(
    neo4j: &neo4rs::Graph,
    version_id: impl Into<String>,
) -> Result<Option<String>, DatabaseError> {
    Ok(triple::find_one(
        neo4j,
        indexer_ids::EDIT_INDEX_ATTRIBUTE,
        version_id,
        indexer_ids::INDEXER_SPACE_ID,
        None,
    )
    .send()
    .await?
    .map(|triple| triple.value.value))
}
