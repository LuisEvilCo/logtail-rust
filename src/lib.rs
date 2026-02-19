use crate::http_client::service;
use crate::http_client::HttpClient;
use crate::http_client::ReqwestClient;
use crate::r#struct::env_config::{EnvConfig, EnvEnum};
use crate::r#struct::log_level::LogLevel;
// re-export LogSchema to make usable by consumer
pub use crate::r#struct::log_schema::LogSchema;
pub mod http_client;
mod r#struct;

pub struct Logger<C: HttpClient = ReqwestClient> {
    env_config: EnvConfig,
    client: C,
}

impl Default for Logger<ReqwestClient> {
    fn default() -> Self {
        let env_config = EnvConfig::default();
        Self {
            env_config,
            client: ReqwestClient,
        }
    }
}

impl Logger<ReqwestClient> {
    pub fn new(app_version: String, verbose: bool) -> Self {
        let env_config = EnvConfig::new(app_version, verbose);
        Self {
            env_config,
            client: ReqwestClient,
        }
    }
}

impl<C: HttpClient> Logger<C> {
    #[cfg(test)]
    pub(crate) fn with_client(env_config: EnvConfig, client: C) -> Self {
        Self { env_config, client }
    }

    pub async fn info(&self, log: LogSchema) {
        let env_config = &self.env_config;
        let better_log = log.to_betterstack(env_config, LogLevel::Info);
        if better_log.env != EnvEnum::Local {
            let _result = service::push_log(&self.client, env_config, &better_log).await;
        }
        if env_config.verbose {
            println!("{}", better_log);
        }
    }

    pub async fn warn(&self, log: LogSchema) {
        let env_config = &self.env_config;
        let better_log = log.to_betterstack(&self.env_config, LogLevel::Warn);
        if better_log.env != EnvEnum::Local {
            let _result = service::push_log(&self.client, env_config, &better_log).await;
        }
        if self.env_config.verbose {
            println!("{}", better_log);
        }
    }

    pub async fn error(&self, log: LogSchema) {
        let env_config = &self.env_config;
        let better_log = log.to_betterstack(&self.env_config, LogLevel::Error);
        if better_log.env != EnvEnum::Local {
            let _result = service::push_log(&self.client, env_config, &better_log).await;
        }
        if self.env_config.verbose {
            eprintln!("{}", better_log);
        }
    }

    pub async fn debug(&self, log: LogSchema) {
        let better_log = log.to_betterstack(&self.env_config, LogLevel::Debug);
        println!("{}", better_log);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_client::mock::MockHttpClient;
    use crate::r#struct::env_config::EnvConfig;
    use std::sync::atomic::Ordering;

    fn qa_config() -> EnvConfig {
        EnvConfig::from_values(
            "1.0.0".to_string(),
            EnvEnum::QA,
            "token".to_string(),
            false,
        )
    }

    fn local_config() -> EnvConfig {
        EnvConfig::from_values(
            "1.0.0".to_string(),
            EnvEnum::Local,
            "token".to_string(),
            false,
        )
    }

    fn test_log() -> LogSchema {
        LogSchema {
            message: "test".to_string(),
            context: "ctx".to_string(),
        }
    }

    #[tokio::test]
    async fn info_sends_info_level() {
        let mock = MockHttpClient::with_success(None);
        let logger = Logger::with_client(qa_config(), mock);

        logger.info(test_log()).await;

        let body = logger.client.captured_body.lock().unwrap().clone().unwrap();
        assert_eq!(body["level"], "Info");
    }

    #[tokio::test]
    async fn warn_sends_warn_level() {
        let mock = MockHttpClient::with_success(None);
        let logger = Logger::with_client(qa_config(), mock);

        logger.warn(test_log()).await;

        let body = logger.client.captured_body.lock().unwrap().clone().unwrap();
        assert_eq!(body["level"], "Warn");
    }

    #[tokio::test]
    async fn error_sends_error_level() {
        let mock = MockHttpClient::with_success(None);
        let logger = Logger::with_client(qa_config(), mock);

        logger.error(test_log()).await;

        let body = logger.client.captured_body.lock().unwrap().clone().unwrap();
        assert_eq!(body["level"], "Error");
    }

    #[tokio::test]
    async fn debug_skips_http() {
        let mock = MockHttpClient::with_success(None);
        let logger = Logger::with_client(qa_config(), mock);

        logger.debug(test_log()).await;

        assert_eq!(logger.client.call_count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn local_env_skips_http() {
        let mock = MockHttpClient::with_success(None);
        let logger = Logger::with_client(local_config(), mock);

        logger.info(test_log()).await;
        logger.warn(test_log()).await;
        logger.error(test_log()).await;

        assert_eq!(logger.client.call_count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn non_local_env_sends_http() {
        let mock = MockHttpClient::with_success(None);
        let logger = Logger::with_client(qa_config(), mock);

        logger.info(test_log()).await;

        assert_eq!(logger.client.call_count.load(Ordering::SeqCst), 1);
    }
}
