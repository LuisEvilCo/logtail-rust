use crate::r#struct::env_config::EnvConfig;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Serialize;
use serde_json::Value;
use std::io::ErrorKind;


pub async fn post<T>(
    url: &str,
    _config: &EnvConfig,
    data: &T,
    extra_headers: Option<HeaderMap>,
) -> Result<Option<Value>, std::io::Error>
where
    T: Serialize,
{
    let mut header_map = HeaderMap::new();

    if let Some(value) = extra_headers {
        // support other http clients
        header_map.extend(value.iter().map(|(k, v)| (k.clone(), v.clone())))
    }

    let json_body = serde_json::to_value(data).expect("Failed to serialize struct to JSON");

    let client = reqwest::Client::new();

    let response = client
        .post(url)
        .headers(build_headers(Some(header_map)))
        .json(&json_body)
        .send()
        .await
        .map_err(|e| std::io::Error::new(ErrorKind::Other, format!("{}", e)))?;

    if response.status().is_success() {
        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, format!("{}", e)))?;

        if !body_bytes.is_empty() {
            let result_value: Value = serde_json::from_slice(&body_bytes)
                .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, format!("{}", e)))?;

            return Ok(Option::from(result_value));
        }

        Ok(None)
    } else {
        match response.status().as_u16() {
            // Handle specific status codes with custom logic
            400 => Err(std::io::Error::new(
                ErrorKind::BrokenPipe,
                "Error getting http result : 400",
            )),
            404 => Err(std::io::Error::new(
                ErrorKind::NotFound,
                "Error getting http result : 404",
            )),
            // Handle other status codes as needed
            code => Err(std::io::Error::new(
                ErrorKind::PermissionDenied,
                format!("Error getting http result : {:?}", code),
            )),
        }
    }
}

fn build_headers(input_headers: Option<HeaderMap>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Some(extra) = input_headers {
        headers = extra;
    }

    headers.insert("Accept", HeaderValue::from_static("application/json"));

    headers
}
