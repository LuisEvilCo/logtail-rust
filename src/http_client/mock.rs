use super::{HttpClient, LogtailError};
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub(crate) struct MockHttpClient {
    result: Mutex<Result<Option<Value>, String>>,
    sequence: Mutex<Option<Vec<Result<Option<Value>, (u16, String)>>>>,
    pub captured_url: Mutex<Option<String>>,
    pub captured_body: Mutex<Option<Value>>,
    pub captured_headers: Mutex<Option<HeaderMap>>,
    pub call_count: AtomicUsize,
}

impl MockHttpClient {
    pub fn with_success(response: Option<Value>) -> Self {
        Self {
            result: Mutex::new(Ok(response)),
            sequence: Mutex::new(None),
            captured_url: Mutex::new(None),
            captured_body: Mutex::new(None),
            captured_headers: Mutex::new(None),
            call_count: AtomicUsize::new(0),
        }
    }

    pub fn with_error(message: &str) -> Self {
        Self {
            result: Mutex::new(Err(message.to_string())),
            sequence: Mutex::new(None),
            captured_url: Mutex::new(None),
            captured_body: Mutex::new(None),
            captured_headers: Mutex::new(None),
            call_count: AtomicUsize::new(0),
        }
    }

    pub fn with_sequence(results: Vec<Result<Option<Value>, (u16, String)>>) -> Self {
        let mut reversed = results;
        reversed.reverse(); // reverse so we can pop from the end
        Self {
            result: Mutex::new(Ok(None)), // fallback, unused when sequence is present
            sequence: Mutex::new(Some(reversed)),
            captured_url: Mutex::new(None),
            captured_body: Mutex::new(None),
            captured_headers: Mutex::new(None),
            call_count: AtomicUsize::new(0),
        }
    }
}

impl HttpClient for MockHttpClient {
    async fn post_json(
        &self,
        url: &str,
        body: &Value,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Option<Value>, LogtailError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        *self.captured_url.lock().unwrap() = Some(url.to_string());
        *self.captured_body.lock().unwrap() = Some(body.clone());
        *self.captured_headers.lock().unwrap() = extra_headers;

        // If we have a sequence, pop the next result
        let mut seq = self.sequence.lock().unwrap();
        if let Some(ref mut results) = *seq {
            if let Some(next) = results.pop() {
                return match next {
                    Ok(val) => Ok(val),
                    Err((status, message)) => Err(LogtailError::Http { status, message }),
                };
            }
        }
        drop(seq);

        // Fall back to single result
        match &*self.result.lock().unwrap() {
            Ok(val) => Ok(val.clone()),
            Err(msg) => Err(LogtailError::Http {
                status: 500,
                message: msg.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_client::service::push_log_with_retry;
    use crate::http_client::RetryConfig;
    use crate::r#struct::betterstack_log_schema::BetterStackLogSchema;
    use crate::r#struct::env_config::{EnvConfig, EnvEnum};
    use crate::r#struct::log_level::LogLevel;
    use std::time::Duration;

    fn test_config() -> EnvConfig {
        EnvConfig::from_values(
            "1.0.0".to_string(),
            EnvEnum::QA,
            "test-token".to_string(),
            false,
        )
    }

    fn test_log() -> BetterStackLogSchema {
        BetterStackLogSchema {
            env: EnvEnum::QA,
            message: "test".to_string(),
            context: "ctx".to_string(),
            level: LogLevel::Info,
            app_version: "1.0.0".to_string(),
        }
    }

    fn fast_retry_config(max_retries: u32) -> RetryConfig {
        RetryConfig {
            max_retries,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            jitter: false,
        }
    }

    #[tokio::test]
    async fn retries_on_5xx_and_succeeds() {
        let mock = MockHttpClient::with_sequence(vec![
            Err((500, "internal server error".to_string())),
            Err((502, "bad gateway".to_string())),
            Ok(Some(serde_json::json!({"status": "ok"}))),
        ]);

        let result =
            push_log_with_retry(&mock, &test_config(), &test_log(), &fast_retry_config(3)).await;

        assert!(result.is_ok());
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn no_retry_on_4xx() {
        let mock = MockHttpClient::with_sequence(vec![Err((400, "bad request".to_string()))]);

        let result =
            push_log_with_retry(&mock, &test_config(), &test_log(), &fast_retry_config(3)).await;

        assert!(result.is_err());
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn no_retry_on_serialization_error() {
        // A 422 is not retryable (status < 500)
        let mock =
            MockHttpClient::with_sequence(vec![Err((422, "unprocessable entity".to_string()))]);

        let result =
            push_log_with_retry(&mock, &test_config(), &test_log(), &fast_retry_config(3)).await;

        assert!(result.is_err());
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn exhausts_retries_and_returns_error() {
        let mock = MockHttpClient::with_sequence(vec![
            Err((500, "fail 1".to_string())),
            Err((500, "fail 2".to_string())),
            Err((500, "fail 3".to_string())),
            Err((500, "fail 4".to_string())),
        ]);

        let result =
            push_log_with_retry(&mock, &test_config(), &test_log(), &fast_retry_config(3)).await;

        assert!(result.is_err());
        // max_retries=3 means 1 initial + 3 retries = 4 attempts
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn no_retry_when_max_retries_zero() {
        let mock = MockHttpClient::with_sequence(vec![Err((500, "server error".to_string()))]);

        let result =
            push_log_with_retry(&mock, &test_config(), &test_log(), &fast_retry_config(0)).await;

        assert!(result.is_err());
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn succeeds_on_first_try_no_retry_needed() {
        let mock = MockHttpClient::with_sequence(vec![Ok(Some(serde_json::json!({"ok": true})))]);

        let result =
            push_log_with_retry(&mock, &test_config(), &test_log(), &fast_retry_config(3)).await;

        assert!(result.is_ok());
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 1);
    }
}
