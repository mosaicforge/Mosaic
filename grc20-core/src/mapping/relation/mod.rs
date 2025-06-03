pub mod delete_many;
pub mod delete_one;
pub mod find_many;
pub mod find_many_to;
pub mod find_one;
pub mod find_one_to;
pub mod insert_many;
pub mod insert_one;
pub mod models;
pub mod utils;

use crate::block::BlockMetadata;

pub use delete_many::DeleteManyQuery;
pub use delete_one::DeleteOneQuery;
pub use find_many::FindManyQuery;
pub use find_many_to::FindManyToQuery;
pub use find_one::FindOneQuery;
pub use find_one_to::FindOneToQuery;
pub use insert_many::InsertManyQuery;
pub use insert_one::InsertOneQuery;
pub use models::{Relation, RelationEdge};
pub use utils::RelationFilter;

pub fn delete_many(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> DeleteManyQuery {
    DeleteManyQuery::new(neo4j, block, space_id.into(), space_version.into())
}

pub fn delete_one(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    relation_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> DeleteOneQuery {
    DeleteOneQuery::new(
        neo4j.clone(),
        block.clone(),
        relation_id.into(),
        space_id.into(),
        space_version.into(),
    )
}

/// Creates a query to find a single relation by its ID and space ID if it exists. Supports optional
/// filtering by version.
///
/// ```rust
/// use grc20_core::mapping::relation;
///
/// // Get current relation
/// let maybe_relation = relation::find_one::<Relation>(&neo4j, "relation_id", "space_id", None)
///     .send()
///     .await?;
///
/// // Get relation in a specific space and version
/// let maybe_relation = relation::find_one::<Relation>(&neo4j, "relation_id", "space_id", Some("space_version".to_string()))
///     .send()
///     .await?;
/// ```
pub fn find_one<T>(
    neo4j: &neo4rs::Graph,
    relation_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
) -> find_one::FindOneQuery<T> {
    FindOneQuery::new(neo4j, relation_id.into(), space_id.into(), space_version)
}

/// Creates a query to find multiple relations. Supports filtering by relation_type and its to/from entities.
/// The results are ordered by relation index.
///
/// See [`RelationFilter`] for more details on filtering options.
///
/// ```rust
/// use grc20_core::mapping::relation;
/// use grc20_core::mapping::query_utils::order_by;
///
/// // Find relations of a specific type (e.g.: "Parent").
/// let relations = relation::find_many::<Relation>(&neo4j)
///     .filter(relation::RelationFilter::default()
///         // Filter by relation type "Parent" (we provide an entity filter with the ID "Parent")
///         .relation_type(entity::EntityFilter::default().id("Parent")))
///     .limit(10)
///     .send()
///     .await?;
///
/// // Find relations with a specific from entity, in this case relations that have a
/// // any type of relation between "Alice" and "Bob".
/// let relations = relation::find_many::<Relation>(&neo4j)
///     .filter(relation::RelationFilter::default()
///         // Filter by from entity with ID "Alice"
///         .from_(entity::EntityFilter::default().id("Alice"))
///         .to_(entity::EntityFilter::default().id("Bob")))
///     .send()
///     .await?;
/// ```
pub fn find_many<T>(neo4j: &neo4rs::Graph) -> FindManyQuery<T> {
    FindManyQuery::new(neo4j)
}

/// Same as `find_one`, but it returns the `to` entity of the relation instead of the
/// relation itself.
pub fn find_one_to<T>(
    neo4j: &neo4rs::Graph,
    relation_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
) -> FindOneToQuery<T> {
    FindOneToQuery::new(neo4j, relation_id.into(), space_id.into(), space_version)
}

/// Same as `find_many`, but it returns the `to` entities of the relations instead of the
/// relations themselves. This is useful when you want to retrieve the target entities of
/// a set of relations without fetching the relations themselves.
pub fn find_many_to<T>(neo4j: &neo4rs::Graph) -> FindManyToQuery<T> {
    FindManyToQuery::new(neo4j)
}

pub fn insert_one<T>(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
    relation: T,
) -> InsertOneQuery<T> {
    InsertOneQuery::new(
        neo4j,
        block,
        space_id.into(),
        space_version.into(),
        relation,
    )
}

pub fn insert_many<T>(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> InsertManyQuery<T> {
    InsertManyQuery::new(neo4j, block, space_id.into(), space_version.into())
}
