mod base_client;
pub mod service;

pub use base_client::ReqwestClient;

use reqwest::header::HeaderMap;
use serde_json::Value;
use std::future::Future;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum LogtailError {
    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String },
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
}

impl LogtailError {
    pub fn is_retryable(&self) -> bool {
        match self {
            LogtailError::Http { status, .. } => *status >= 500,
            LogtailError::Network(_) => true,
            LogtailError::Serialization(_) => false,
        }
    }
}

pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5),
            jitter: true,
        }
    }
}

pub trait HttpClient: Send + Sync {
    fn post_json(
        &self,
        url: &str,
        body: &Value,
        extra_headers: Option<HeaderMap>,
    ) -> impl Future<Output = Result<Option<Value>, LogtailError>> + Send;
}

#[cfg(test)]
pub(crate) mod mock;
