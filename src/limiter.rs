use std::sync::RwLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub window_seconds: u64,
    pub request_per_window: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            window_seconds: 60,
            request_per_window: 100,
        }
    }
}


#[derive(Default)]
pub struct RateLimiter {
    config: RwLock<RateLimitConfig>,
    // TODO: per-client counters, e.g. DashMap<String, ClientWindow>,
    // or Mutex<HashMap<String, ClientWindow>>
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self { config: RwLock::new(config) }
    }

    pub fn config(&self) -> RateLimitConfig {
        // Because RateLimitConfig impls Copy, the deref returns a copy
        *self.config.read().unwrap()
    }

    pub fn update_config(&self, config: RateLimitConfig) {   // &self, not &mut self
        *self.config.write().unwrap() = config;
    }

    pub fn check(&self, client_id: &str, resource: &str) -> (bool, u64, Option<u64>) {
        let config = self.config();
        // update this client's counters under their own fine-grained lock
        (true, config.request_per_window, Some(100))
    }
}
