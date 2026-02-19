use crate::http_client::base_client;
use crate::r#struct::betterstack_log_schema::BetterStackLogSchema;
use crate::r#struct::env_config::EnvConfig;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;

/// Pushes a log to the BetterStack logs server asynchronously and returns a value.
///
/// # Arguments
///
/// * `config` - The configuration of the server.
/// * `log` - The log to be pushed.
///
/// # Returns
///
/// * If the log is sent successfully, returns `Some` containing the continuation value.
/// * If there is an error sending the log, prints the error message and returns `None`.
pub async fn push_log(config: &EnvConfig, log: &BetterStackLogSchema) -> Option<Value> {
    let logs_url = "https://in.logs.betterstack.com";
    let bearer_header = bearer_headers(config);

    let http_result = base_client::post(logs_url, log, Some(bearer_header)).await;

    match http_result {
        Err(err) => {
            println!("!!! Error sending log : {}", err);
            // Ignore the error sending logs, so we can continue
            // logging errors must not crash the app
            None
        }
        Ok(continuation_value) => Some(continuation_value?),
    }
}

/// Generate a bearer header for the given server configuration.
///
/// # Parameters
/// - `server_config`: A reference to the server configuration.
///
/// # Returns
/// The generated bearer header as a `HeaderMap`.
///
fn bearer_headers(config: &EnvConfig) -> HeaderMap {
    let logs_source_token = config.logs_source_token.as_str();
    let bearer_value_str = format!("Bearer {}", logs_source_token);
    let bearer_value = &bearer_value_str;

    let mut headers = HeaderMap::new();

    headers.insert(
        "Authorization",
        HeaderValue::from_str(bearer_value).unwrap(),
    );
    headers.insert(
        "Content-Type",
        HeaderValue::from_str("application/json").unwrap(),
    );

    headers
}
