# Logtail Rust

Logtail rust is an http wrapper for sending logs to [betterstack](https://betterstack.com/logs)

## Environment Variables

| Variable            | Description                                                                                                              |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| ENVIRONMENT         | Can be "local", "qa", "preprod" or "prod"                                                                                |
| LOGS_SOURCE_TOKEN   | [Docs](https://betterstack.com/docs/logs/logging-start/#step-2-test-the-pipes)                                           |

## Usage

### Basic usage

```rust
let logger = Logger::new(env!("CARGO_PKG_VERSION").to_string(), true);

let log = logtail_rust::LogSchema {
    message: "Server started".to_string(),
    context: format!("{} - {}", file!(), line!()),
};

// Log methods now return Result<(), LogtailError>
logger.info(log).await?;
```

### Builder pattern

```rust
use logtail_rust::{Logger, LogSchema};
use std::time::Duration;

let logger = Logger::builder()
    .app_version("1.0.0")
    .verbose(true)
    .max_retries(3)
    .base_delay(Duration::from_millis(200))
    .max_delay(Duration::from_millis(800))
    .jitter(true)
    .build();

logger.info(LogSchema {
    message: "hello".to_string(),
    context: "main".to_string(),
}).await?;
```

### Custom retry config

```rust
use logtail_rust::http_client::RetryConfig;
use std::time::Duration;

let retry = RetryConfig {
    max_retries: 3,
    base_delay: Duration::from_millis(500),
    max_delay: Duration::from_millis(3000),
    jitter: true,
};

let logger = Logger::with_retry(retry);
```

### Default (reads from env vars, default retry)

```rust
let logger = Logger::default();
```

## Configuration Reference

### Logger options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `app_version` | `String` | `CARGO_PKG_VERSION` | Version string included in every log sent to BetterStack |
| `verbose` | `bool` | `true` | When `true`, logs are also printed to stdout (`info`/`warn`/`debug`) or stderr (`error`) in addition to being sent over HTTP |
| `environment` | `EnvEnum` | Read from `ENVIRONMENT` env var | One of `Local`, `QA`, `PreProd`, `Prod`. When set to `Local`, HTTP calls are skipped entirely |
| `logs_source_token` | `String` | Read from `LOGS_SOURCE_TOKEN` env var | BetterStack source token for authentication |

### Retry options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `max_retries` | `u32` | `3` | Maximum number of retry attempts after the initial call fails |
| `base_delay` | `Duration` | `1000ms` | Initial delay before the first retry, doubled on each subsequent attempt |
| `max_delay` | `Duration` | `5000ms` | Upper bound on the backoff delay |
| `jitter` | `bool` | `true` | Randomize the delay (0 to computed delay) to avoid thundering herd |

## Retry Behavior

Failed HTTP requests are automatically retried with exponential backoff. Only transient errors are retried:

- **5xx HTTP errors** — retried (server errors)
- **Network errors** — retried (connection failures, timeouts)
- **4xx HTTP errors** — not retried (client errors)
- **Serialization errors** — not retried

## Error Handling

All log methods (`info`, `warn`, `error`, `debug`) return `Result<(), LogtailError>`. Errors are no longer silently swallowed.

`LogtailError` variants:
- `Http { status, message }` — HTTP error with status code
- `Serialization(serde_json::Error)` — JSON serialization failure
- `Network(reqwest::Error)` — Network/connection error

## Migration from 0.2.x

- **Breaking:** `info()`, `warn()`, `error()`, `debug()` now return `Result<(), LogtailError>` instead of `()`
- **Breaking:** `push_log()` now returns `Result<Option<Value>, LogtailError>` instead of `Option<Value>`
- Add `.await?` or `let _ =` to log calls to handle the new return type
- `tokio` with `time` feature is now a required dependency
- New `Logger::builder()`, `Logger::with_retry()` constructors available
