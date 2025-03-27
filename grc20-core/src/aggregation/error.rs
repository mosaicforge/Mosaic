use crate::error::DatabaseError;

#[derive(Debug, thiserror::Error)]
pub enum AggregationError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
}
