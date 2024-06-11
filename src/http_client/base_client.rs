use crate::r#struct::env_config::EnvConfig;
use reqwest::header::HeaderMap;
use serde::Serialize;

#[allow(dead_code)]
pub async fn post<T>(_url: &str, _config: &EnvConfig, data: &T, extra_headers: Option<HeaderMap>)
where
    T: Serialize,
{
    let mut header_map = HeaderMap::new();

    if let Some(value) = extra_headers {
        // support other http clients
        header_map.extend(value.iter().map(|(k, v)| (k.clone(), v.clone())))
    }

    let _json_body = serde_json::to_value(data).expect("Failed to serialize struct to JSON");

    let _client = reqwest::Client::new();

    todo!()
}
