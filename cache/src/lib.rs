mod config;
mod error;

use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::{debug, error};

pub use config::CacheConfig;
pub use error::CacheError;

pub struct KgCache {
    client: memcache::Client,
    default_expiry: Option<Duration>,
}

impl KgCache {
    pub fn new(config: CacheConfig) -> Result<Self, CacheError> {
        let client = memcache::connect(vec![config.server_list().as_str()])?;
        Ok(Self {
            client,
            default_expiry: config.default_expiry,
        })
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        debug!("Getting value for key: {}", key);
        match self.client.get::<String>(key)? {
            Some(data) => match serde_json::from_str(&data) {
                Ok(value) => Ok(Some(value)),
                Err(e) => {
                    error!("Failed to deserialize value for key {}: {}", key, e);
                    Err(CacheError::Serialization(e.to_string()))
                }
            },
            None => Ok(None),
        }
    }

    pub fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        expiry: Option<Duration>,
    ) -> Result<(), CacheError> {
        debug!("Setting value for key: {}", key);
        let json =
            serde_json::to_string(value).map_err(|e| CacheError::Serialization(e.to_string()))?;

        let expiry = expiry.or(self.default_expiry);
        let expiry_secs = expiry.map(|d| d.as_secs() as u32).unwrap_or(0);
        self.client.set(key, &json, expiry_secs)?;
        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<bool, CacheError> {
        debug!("Deleting key: {}", key);
        Ok(self.client.delete(key)?)
    }

    pub fn flush(&self) -> Result<(), CacheError> {
        debug!("Flushing cache");
        Ok(self.client.flush()?)
    }

    pub fn increment(&self, key: &str, amount: u64) -> Result<u64, CacheError> {
        debug!("Incrementing key {} by {}", key, amount);
        Ok(self.client.increment(key, amount)?)
    }

    pub fn decrement(&self, key: &str, amount: u64) -> Result<u64, CacheError> {
        debug!("Decrementing key {} by {}", key, amount);
        Ok(self.client.decrement(key, amount)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        field1: String,
        field2: i32,
    }

    fn create_test_cache() -> KgCache {
        let config = CacheConfig::new(vec!["localhost:11211".to_string()]);
        KgCache::new(config).expect("Failed to create test cache")
    }

    #[test]
    fn test_set_get() {
        let cache = create_test_cache();
        let test_struct = TestStruct {
            field1: "test".to_string(),
            field2: 42,
        };

        cache.set("test_key", &test_struct, None).unwrap();
        let result: Option<TestStruct> = cache.get("test_key").unwrap();
        assert_eq!(result.unwrap(), test_struct);
    }

    #[test]
    fn test_delete() {
        let cache = create_test_cache();
        let test_struct = TestStruct {
            field1: "test".to_string(),
            field2: 42,
        };

        cache.set("test_key", &test_struct, None).unwrap();
        assert!(cache.delete("test_key").unwrap());
        let result: Option<TestStruct> = cache.get("test_key").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_expiry() {
        let cache = create_test_cache();
        let test_struct = TestStruct {
            field1: "test".to_string(),
            field2: 42,
        };

        cache
            .set("test_key", &test_struct, Some(Duration::from_secs(1)))
            .unwrap();
        std::thread::sleep(Duration::from_secs(2));
        let result: Option<TestStruct> = cache.get("test_key").unwrap();
        assert!(result.is_none());
    }
}
