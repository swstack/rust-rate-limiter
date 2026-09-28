use std::time::{Duration, Instant};

use ratelimiter::limiter::{RateLimitConfig, RateLimiter};

fn limiter(window_seconds: u64, request_per_window: u64) -> RateLimiter {
    RateLimiter::new(RateLimitConfig {
        window_seconds,
        request_per_window,
    })
}

#[test]
fn allows_up_to_limit_then_rejects() {
    let limiter = limiter(10, 3);
    let now = Instant::now();

    assert_eq!(limiter.check_at("a", "/", now), (true, 2, None));
    assert_eq!(limiter.check_at("a", "/", now), (true, 1, None));
    assert_eq!(limiter.check_at("a", "/", now), (true, 0, None));
    assert_eq!(limiter.check_at("a", "/", now), (false, 0, Some(10)));
}

#[test]
fn retry_after_counts_down_and_rounds_up() {
    let limiter = limiter(10, 1);
    let start = Instant::now();
    limiter.check_at("a", "/", start);

    let (_, _, retry_after) = limiter.check_at("a", "/", start + Duration::from_millis(3_500));
    assert_eq!(retry_after, Some(7)); // 6.5s left, rounded up
}

#[test]
fn window_resets_after_expiry() {
    let limiter = limiter(10, 1);
    let start = Instant::now();

    assert!(limiter.check_at("a", "/", start).0);
    assert!(!limiter.check_at("a", "/", start + Duration::from_secs(9)).0);
    assert_eq!(
        limiter.check_at("a", "/", start + Duration::from_secs(10)),
        (true, 0, None)
    );
}

#[test]
fn clients_are_limited_independently() {
    let limiter = limiter(10, 1);
    let now = Instant::now();

    assert!(limiter.check_at("a", "/", now).0);
    assert!(!limiter.check_at("a", "/", now).0);
    assert!(limiter.check_at("b", "/", now).0);
}

#[test]
fn config_update_applies_to_next_check() {
    let limiter = limiter(10, 1);
    let now = Instant::now();

    assert!(limiter.check_at("a", "/", now).0);
    assert!(!limiter.check_at("a", "/", now).0);

    limiter.update_config(RateLimitConfig {
        window_seconds: 10,
        request_per_window: 5,
    });
    assert_eq!(limiter.check_at("a", "/", now), (true, 3, None));
}

#[test]
fn resources_are_limited_independently() {
    let limiter = limiter(10, 1);
    let now = Instant::now();

    assert!(limiter.check_at("a", "/search", now).0);
    assert!(!limiter.check_at("a", "/search", now).0);
    assert!(limiter.check_at("a", "/login", now).0);
}
