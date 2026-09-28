# Rate Limiter

Simple rate limiter service in Rust, built on [axum](https://github.com/tokio-rs/axum). All state is held in memory; no external services are required.

## Run the application

```
cargo run --release
```

The server listens on `0.0.0.0:3000` by default. Set `BIND_ADDR` to change it:

```
BIND_ADDR=127.0.0.1:8080 cargo run --release
```

## API

**`POST /api/v1/ratelimit`**: check whether a request should be allowed.

```
curl -X POST http://127.0.0.1:3000/api/v1/ratelimit \
  -H 'content-type: application/json' \
  -d '{"client_id": "abc", "resource": "/foo"}'
```

```json
{"allowed": true, "remaining": 99}
```

When the limit is exceeded, `allowed` is `false` and `retry_after` gives the seconds until the window resets:

```json
{"allowed": false, "remaining": 0, "retry_after": 42}
```

**`POST /api/v1/configure`**: set the rate limit configuration (default: 100 requests per 60 seconds).

```
curl -X POST http://127.0.0.1:3000/api/v1/configure \
  -H 'content-type: application/json' \
  -d '{"window_seconds": 10, "request_per_window": 5}'
```

```json
{"window_seconds": 10, "request_per_window": 5}
```

**`GET /api/v1/config`**: return the current configuration.

## Run the tests

```
cargo test
```

- `tests/functional.rs`: API behaviour (routing, request/response shapes, config persistence).
- `tests/limiter.rs`: the rate limiting algorithm, using an injected clock so window expiry is tested without sleeping.
- `tests/nonfunctional.rs`: latency and throughput against the real server over TCP.

## Design Tradeoffs

Fixed window rate limiting:
- This is a naive approach that was simpler to implement but allows for bursty traffic. A client can send up to 2x the limit around a window boundary. A sliding window or token bucket would fix this.

Limits keyed on client ID and resource:
- The spec is ambiguous here. The downside of this approach is that a global limit per client is not enforced, only per resource+client. Limiting per client only is a one-line change.

Single global config:
- All clients have the same window size and limit. Config changes apply immediately, including to windows already in progress.

Single lock for all clients:
- All counters live in one `Mutex<HashMap>`. It is simple and the lock is held very briefly, but every check contends on it. A sharded map (e.g. `DashMap`) would reduce contention.

No client eviction:
- We never clean up or evict clients from our in-memory limiter, so memory grows with every client/resource seen.

Memory-backed:
- The limiter state is stored in memory, so it will be lost on restart and can't be shared across instances. I would use Redis to implement a persistent, shared limiter.

## Load Testing

**Load testing from tests/nonfunctional.rs**:

```
cargo test --release --test nonfunctional -- --nocapture
```

This sends 1000 requests from 50 concurrent workers to the real server over TCP, and asserts at least 1000 rps and a p99 latency under 10ms. Typical output on an Apple M3 Pro (36 GB):

```
1000 requests in 14.1ms (70774 rps), p50 541µs, p99 944µs
```

The client and server share one process and CPU, so treat this as a smoke test rather than a benchmark.

**Use real load-testing tool [oha](https://github.com/hatoo/oha)**:

Start the server:

```
cargo run --release
```

In another terminal, raise the rate limit so requests aren't rejected (every `oha` request uses the same client and resource):

```
curl -X POST http://127.0.0.1:3000/api/v1/configure \
  -H 'content-type: application/json' \
  -d '{"window_seconds": 60, "request_per_window": 100000000}'
```

Then run the load test:

```
oha -z 30s -c 50 -m POST \
  -H 'content-type: application/json' \
  -d '{"client_id": "abc", "resource": "/foo"}' \
  http://127.0.0.1:3000/api/v1/ratelimit
```

Note: the `--release` flag compiles the binary with optimizations enabled and debug info removed, which is much faster than the default debug build. Always load test a release build.
