use super::HttpClient;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use std::io::ErrorKind;

pub struct ReqwestClient;

impl HttpClient for ReqwestClient {
    async fn post_json(
        &self,
        url: &str,
        body: &Value,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Option<Value>, std::io::Error> {
        let mut header_map = HeaderMap::new();

        if let Some(value) = extra_headers {
            header_map.extend(value.iter().map(|(k, v)| (k.clone(), v.clone())))
        }

        let client = reqwest::Client::new();

        let response = client
            .post(url)
            .headers(build_headers(Some(header_map)))
            .json(body)
            .send()
            .await
            .map_err(|e| std::io::Error::other(format!("{}", e)))?;

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
                400 => Err(std::io::Error::new(
                    ErrorKind::BrokenPipe,
                    "Error getting http result : 400",
                )),
                404 => Err(std::io::Error::new(
                    ErrorKind::NotFound,
                    "Error getting http result : 404",
                )),
                code => Err(std::io::Error::new(
                    ErrorKind::PermissionDenied,
                    format!("Error getting http result : {:?}", code),
                )),
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_headers_none_adds_accept() {
        let headers = build_headers(None);
        assert_eq!(
            headers.get("Accept").unwrap(),
            HeaderValue::from_static("application/json")
        );
        assert_eq!(headers.len(), 1);
    }

    #[test]
    fn build_headers_merges_extra_headers() {
        let mut extra = HeaderMap::new();
        extra.insert("Authorization", HeaderValue::from_static("Bearer token123"));

        let headers = build_headers(Some(extra));
        assert_eq!(
            headers.get("Authorization").unwrap(),
            HeaderValue::from_static("Bearer token123")
        );
        assert_eq!(
            headers.get("Accept").unwrap(),
            HeaderValue::from_static("application/json")
        );
        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn build_headers_accept_overwrites_extra() {
        let mut extra = HeaderMap::new();
        extra.insert("Accept", HeaderValue::from_static("text/plain"));

        let headers = build_headers(Some(extra));
        assert_eq!(
            headers.get("Accept").unwrap(),
            HeaderValue::from_static("application/json")
        );
    }
}
