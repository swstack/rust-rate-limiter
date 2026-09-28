use std::collections::HashMap;
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};

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


/// A client's current fixed window for one resource: when it started and how many requests it has used.
struct ClientWindow {
    start: Instant,
    count: u64,
}

#[derive(Default)]
pub struct RateLimiter {
    config: RwLock<RateLimitConfig>,
    // One lock for all clients; simple, but every check contends on it.
    // TODO: swap for DashMap<(String, String), ClientWindow> to shard the lock.
    // Keyed by (client_id, resource).
    clients: Mutex<HashMap<(String, String), ClientWindow>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config: RwLock::new(config),
            clients: Mutex::new(HashMap::new()),
        }
    }

    pub fn config(&self) -> RateLimitConfig {
        // Because RateLimitConfig impls Copy, the deref returns a copy
        *self.config.read().unwrap()
    }

    pub fn update_config(&self, config: RateLimitConfig) {
        *self.config.write().unwrap() = config;
    }

    pub fn check(&self, client_id: &str, resource: &str) -> (bool, u64, Option<u64>) {
        // TODO: Clients never expire or are never removed from the map.
        self.check_at(client_id, resource, Instant::now())
    }

    /// Same as `check`, but with the current time passed in so tests can control the clock.
    pub fn check_at(
        &self,
        client_id: &str,
        resource: &str,
        now: Instant,
    ) -> (bool, u64, Option<u64>) {
        let config = self.config();
        let window = Duration::from_secs(config.window_seconds);

        let mut clients = self.clients.lock().unwrap();
        let entry = clients
            .entry((client_id.to_string(), resource.to_string()))
            .or_insert(ClientWindow { start: now, count: 0 });

        // The window has expired, so start a fresh one.
        if now.duration_since(entry.start) >= window {
            *entry = ClientWindow { start: now, count: 0 };
        }

        if entry.count < config.request_per_window {
            entry.count += 1;
            (true, config.request_per_window - entry.count, None)
        } else {
            let wait = window - now.duration_since(entry.start);
            (false, 0, Some(wait.as_millis().div_ceil(1000) as u64))
        }
    }
}
