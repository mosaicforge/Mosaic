#[derive(Debug, thiserror::Error)]
pub enum TriplesConversionError {
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Missing id")]
    MissingId,
}
