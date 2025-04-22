use std::time::Duration;

#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub servers: Vec<String>,
    pub default_expiry: Option<Duration>,
}

impl CacheConfig {
    pub fn new(servers: Vec<String>) -> Self {
        Self {
            servers,
            default_expiry: None,
        }
    }

    pub fn with_default_expiry(mut self, expiry: Duration) -> Self {
        self.default_expiry = Some(expiry);
        self
    }

    pub fn server_list(&self) -> String {
        self.servers.join(",")
    }
}
