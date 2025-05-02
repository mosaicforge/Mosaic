pub mod delete_many;
pub mod delete_one;
pub mod find_many;
pub mod find_one;
pub mod insert_many;
pub mod insert_one;
pub mod models;
pub mod utils;

pub use delete_one::DeleteOneQuery;
pub use find_many::FindManyQuery;
pub use find_one::FindOneQuery;
pub use insert_one::InsertOneQuery;
pub use models::{Entity, EntityNode, EntityNodeRef, SystemProperties};
pub use utils::{EntityFilter, EntityRelationFilter};

use crate::block::BlockMetadata;

pub fn delete_one(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> DeleteOneQuery {
    DeleteOneQuery::new(
        neo4j,
        block,
        entity_id.into(),
        space_id.into(),
        space_version.into(),
    )
}

pub fn find_one<T>(neo4j: &neo4rs::Graph, id: impl Into<String>) -> FindOneQuery<T> {
    FindOneQuery::new(neo4j, id.into())
}

pub fn find_many<T>(neo4j: &neo4rs::Graph) -> FindManyQuery<T> {
    FindManyQuery::new(neo4j)
}

pub fn insert_one<T>(
    neo4j: &neo4rs::Graph,
    block: &BlockMetadata,
    entity: T,
    space_id: impl Into<String>,
    space_version: impl Into<String>,
) -> InsertOneQuery<T> {
    InsertOneQuery::new(
        neo4j.clone(),
        block.clone(),
        entity,
        space_id.into(),
        space_version.into(),
    )
}
