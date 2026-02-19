use super::{HttpClient, LogtailError, RetryConfig};
use crate::r#struct::betterstack_log_schema::BetterStackLogSchema;
use crate::r#struct::env_config::EnvConfig;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use std::time::Duration;

/// Pushes a log to the BetterStack logs server asynchronously and returns a value.
///
/// # Arguments
///
/// * `client` - The HTTP client to use for sending the request.
/// * `config` - The configuration of the server.
/// * `log` - The log to be pushed.
///
/// # Returns
///
/// * `Ok(Some(value))` if the log is sent successfully and a response body is returned.
/// * `Ok(None)` if the log is sent successfully but no response body is returned.
/// * `Err(LogtailError)` if there is an error sending the log.
pub async fn push_log(
    client: &impl HttpClient,
    config: &EnvConfig,
    log: &BetterStackLogSchema,
) -> Result<Option<Value>, LogtailError> {
    let logs_url = "https://in.logs.betterstack.com";
    let bearer_header = bearer_headers(config);
    let body = serde_json::to_value(log)?;

    client.post_json(logs_url, &body, Some(bearer_header)).await
}

/// Pushes a log with automatic retry on transient failures.
///
/// Uses exponential backoff with optional jitter. Only retries on errors
/// where `is_retryable()` returns true (5xx HTTP errors and network errors).
pub async fn push_log_with_retry(
    client: &impl HttpClient,
    config: &EnvConfig,
    log: &BetterStackLogSchema,
    retry_config: &RetryConfig,
) -> Result<Option<Value>, LogtailError> {
    let mut last_err = None;

    for attempt in 0..=retry_config.max_retries {
        match push_log(client, config, log).await {
            Ok(val) => return Ok(val),
            Err(err) => {
                if !err.is_retryable() || attempt == retry_config.max_retries {
                    return Err(err);
                }
                last_err = Some(err);

                let base_ms = retry_config
                    .base_delay
                    .as_millis()
                    .saturating_mul(2u128.saturating_pow(attempt))
                    as u64;
                let capped_ms = base_ms.min(retry_config.max_delay.as_millis() as u64);

                let delay_ms = if retry_config.jitter && capped_ms > 0 {
                    // Cheap jitter using timestamp nanos
                    let nanos = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .subsec_nanos() as u64;
                    nanos % capped_ms
                } else {
                    capped_ms
                };

                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }

    Err(last_err.unwrap_or_else(|| LogtailError::Http {
        status: 500,
        message: "retry exhausted".to_string(),
    }))
}

/// Generate a bearer header for the given server configuration.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_client::mock::MockHttpClient;
    use crate::r#struct::env_config::{EnvConfig, EnvEnum};
    use crate::r#struct::log_level::LogLevel;
    use std::sync::atomic::Ordering;

    fn test_config() -> EnvConfig {
        EnvConfig::from_values(
            "1.0.0".to_string(),
            EnvEnum::QA,
            "test-source-token".to_string(),
            false,
        )
    }

    fn test_log() -> BetterStackLogSchema {
        BetterStackLogSchema {
            env: EnvEnum::QA,
            message: "test message".to_string(),
            context: "test context".to_string(),
            level: LogLevel::Info,
            app_version: "1.0.0".to_string(),
        }
    }

    #[tokio::test]
    async fn calls_correct_url() {
        let mock = MockHttpClient::with_success(None);
        let _ = push_log(&mock, &test_config(), &test_log()).await;

        let url = mock.captured_url.lock().unwrap().clone().unwrap();
        assert_eq!(url, "https://in.logs.betterstack.com");
    }

    #[tokio::test]
    async fn sends_bearer_header() {
        let mock = MockHttpClient::with_success(None);
        let _ = push_log(&mock, &test_config(), &test_log()).await;

        let headers = mock.captured_headers.lock().unwrap().clone().unwrap();
        assert_eq!(
            headers.get("Authorization").unwrap().to_str().unwrap(),
            "Bearer test-source-token"
        );
    }

    #[tokio::test]
    async fn sends_content_type_json() {
        let mock = MockHttpClient::with_success(None);
        let _ = push_log(&mock, &test_config(), &test_log()).await;

        let headers = mock.captured_headers.lock().unwrap().clone().unwrap();
        assert_eq!(
            headers.get("Content-Type").unwrap().to_str().unwrap(),
            "application/json"
        );
    }

    #[tokio::test]
    async fn sends_serialized_log_body() {
        let mock = MockHttpClient::with_success(None);
        let _ = push_log(&mock, &test_config(), &test_log()).await;

        let body = mock.captured_body.lock().unwrap().clone().unwrap();
        assert_eq!(body["message"], "test message");
        assert_eq!(body["context"], "test context");
        assert_eq!(body["level"], "Info");
        assert_eq!(body["env"], "QA");
        assert_eq!(body["app_version"], "1.0.0");
    }

    #[tokio::test]
    async fn returns_some_on_success() {
        let response = serde_json::json!({"status": "ok"});
        let mock = MockHttpClient::with_success(Some(response.clone()));

        let result = push_log(&mock, &test_config(), &test_log()).await;
        assert_eq!(result.unwrap().unwrap(), response);
    }

    #[tokio::test]
    async fn returns_error_on_failure() {
        let mock = MockHttpClient::with_error("connection refused");

        let result = push_log(&mock, &test_config(), &test_log()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn returns_none_on_empty_body() {
        let mock = MockHttpClient::with_success(None);

        let result = push_log(&mock, &test_config(), &test_log()).await;
        assert!(result.unwrap().is_none());
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 1);
    }
}
