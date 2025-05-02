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

use crate::{block::BlockMetadata, mapping::entity::EntityNodeRef};

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

pub fn find_one<T>(
    neo4j: &neo4rs::Graph,
    relation_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
) -> find_one::FindOneQuery<T> {
    FindOneQuery::new(neo4j, relation_id.into(), space_id.into(), space_version)
}

pub fn find_many<T>(neo4j: &neo4rs::Graph) -> FindManyQuery<T> {
    FindManyQuery::new(neo4j)
}

pub fn find_one_to<T>(
    neo4j: &neo4rs::Graph,
    relation_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
) -> FindOneToQuery<T> {
    FindOneToQuery::new(neo4j, relation_id.into(), space_id.into(), space_version)
}

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
