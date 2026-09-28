# Rate Limiter

Simple rate limiter implementation in Rust. Runs against in memory or configurable Redis backend.


## Run the application

```
cargo run
```

## Design Tradeoffs

Fixed window rate limiting:
- This is a naive approach that was simpler to implement but allows for bursty traffic.

## Load Testing

**Load testing from tests/nonfunctional.rs**:

todo

**Use real load-testing tool [oha](https://github.com/hatoo/oha)**:

First configure the app to raise the rate limit:

```
curl -X POST http://127.0.0.1:3000/api/v1/configure \
  -H 'content-type: application/json' \
  -d '{"client_id":"abc","resource":"/foo"}'
```

Then run the load test:

```
cargo run --release &
oha -z 30s -c 50 -m POST \
  -H 'content-type: application/json' \
  -d '{"window_seconds": 1,"request_per_window": 2000}' \
  http://127.0.0.1:3000/api/v1/ratelimit
```

Note: `--release` flag to compile the binary in release mode (faster binary). Compiler optimizations, debug symbols are disabled, etc.
