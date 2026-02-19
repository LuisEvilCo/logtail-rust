use super::{HttpClient, LogtailError};
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub(crate) struct MockHttpClient {
    result: Mutex<Result<Option<Value>, String>>,
    pub captured_url: Mutex<Option<String>>,
    pub captured_body: Mutex<Option<Value>>,
    pub captured_headers: Mutex<Option<HeaderMap>>,
    pub call_count: AtomicUsize,
}

impl MockHttpClient {
    pub fn with_success(response: Option<Value>) -> Self {
        Self {
            result: Mutex::new(Ok(response)),
            captured_url: Mutex::new(None),
            captured_body: Mutex::new(None),
            captured_headers: Mutex::new(None),
            call_count: AtomicUsize::new(0),
        }
    }

    pub fn with_error(message: &str) -> Self {
        Self {
            result: Mutex::new(Err(message.to_string())),
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

        match &*self.result.lock().unwrap() {
            Ok(val) => Ok(val.clone()),
            Err(msg) => Err(LogtailError::Http {
                status: 500,
                message: msg.clone(),
            }),
        }
    }
}
