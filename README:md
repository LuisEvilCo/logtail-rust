# Logtail Rust

Logtail rust is an http wrapper for sending logs to [betterstack](https://betterstack.com/logs)

Requires the following variables :




| Syntax      | Description |
| ----------- | ----------- |
| ENVIRONMENT      | Can be "local" , "qa", "preprod" or "prod"       |
| LOGS_SOURCE_TOKEN   | [Docs](https://betterstack.com/docs/logs/logging-start/#step-2-test-the-pipes)        |

```rust
// recommended way to instance
let logger = Logger::new(env!("CARGO_PKG_VERSION").to_string(), true);

// it also has a default impl
let _default_logger = Logger::default();

// setup your log message into the LogSchema struct
let bind_address = "192.168.0.1:8000";
let start_message = format!("🚀 Server started successfully {}", &bind_address);

let log = logtail_rust::LogSchema {
    message: start_message,
    context: format!("{} - {}", file!(), line!()),
};

// send your log
logger.info(log).await;
```
