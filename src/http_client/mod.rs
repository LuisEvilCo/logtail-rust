mod base_client;
pub mod service;

pub use base_client::ReqwestClient;

use reqwest::header::HeaderMap;
use serde_json::Value;
use std::future::Future;

pub trait HttpClient: Send + Sync {
    fn post_json(
        &self,
        url: &str,
        body: &Value,
        extra_headers: Option<HeaderMap>,
    ) -> impl Future<Output = Result<Option<Value>, std::io::Error>> + Send;
}

#[cfg(test)]
pub(crate) mod mock;