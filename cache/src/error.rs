use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Memcache error: {0}")]
    Memcache(#[from] memcache::MemcacheError),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Value not found for key: {0}")]
    NotFound(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}
