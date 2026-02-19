mod base_client;
pub mod service;

pub use base_client::ReqwestClient;

use reqwest::header::HeaderMap;
use serde_json::Value;
use std::future::Future;

#[derive(Debug, thiserror::Error)]
pub enum LogtailError {
    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String },
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
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
