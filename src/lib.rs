use crate::http_client::service;
use crate::http_client::HttpClient;
use crate::http_client::ReqwestClient;
use crate::r#struct::env_config::EnvConfig;
use crate::r#struct::log_level::LogLevel;
use std::time::Duration;
// re-export types to make usable by consumers
pub use crate::http_client::{LogtailError, RetryConfig};
pub use crate::r#struct::env_config::EnvEnum;
pub use crate::r#struct::log_schema::LogSchema;
pub mod http_client;
mod r#struct;

pub struct Logger<C: HttpClient = ReqwestClient> {
    env_config: EnvConfig,
    client: C,
    retry_config: RetryConfig,
}

impl Default for Logger<ReqwestClient> {
    fn default() -> Self {
        let env_config = EnvConfig::default();
        Self {
            env_config,
            client: ReqwestClient,
            retry_config: RetryConfig::default(),
        }
    }
}

impl Logger<ReqwestClient> {
    pub fn new(app_version: String, verbose: bool) -> Self {
        let env_config = EnvConfig::new(app_version, verbose);
        Self {
            env_config,
            client: ReqwestClient,
            retry_config: RetryConfig::default(),
        }
    }

    pub fn with_retry(retry_config: RetryConfig) -> Self {
        let env_config = EnvConfig::default();
        Self {
            env_config,
            client: ReqwestClient,
            retry_config,
        }
    }

    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::default()
    }
}

impl<C: HttpClient> Logger<C> {
    #[cfg(test)]
    pub(crate) fn env_config(&self) -> &EnvConfig {
        &self.env_config
    }

    #[cfg(test)]
    pub(crate) fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }

    #[cfg(test)]
    pub(crate) fn with_client(env_config: EnvConfig, client: C) -> Self {
        Self {
            env_config,
            client,
            retry_config: RetryConfig::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_client_and_retry(
        env_config: EnvConfig,
        client: C,
        retry_config: RetryConfig,
    ) -> Self {
        Self {
            env_config,
            client,
            retry_config,
        }
    }

    pub async fn info(&self, log: LogSchema) -> Result<(), LogtailError> {
        let env_config = &self.env_config;
        let better_log = log.to_betterstack(env_config, LogLevel::Info);
        if better_log.env != EnvEnum::Local {
            service::push_log_with_retry(&self.client, env_config, &better_log, &self.retry_config)
                .await?;
        }
        if env_config.verbose {
            println!("{}", better_log);
        }
        Ok(())
    }

    pub async fn warn(&self, log: LogSchema) -> Result<(), LogtailError> {
        let env_config = &self.env_config;
        let better_log = log.to_betterstack(&self.env_config, LogLevel::Warn);
        if better_log.env != EnvEnum::Local {
            service::push_log_with_retry(&self.client, env_config, &better_log, &self.retry_config)
                .await?;
        }
        if self.env_config.verbose {
            println!("{}", better_log);
        }
        Ok(())
    }

    pub async fn error(&self, log: LogSchema) -> Result<(), LogtailError> {
        let env_config = &self.env_config;
        let better_log = log.to_betterstack(&self.env_config, LogLevel::Error);
        if better_log.env != EnvEnum::Local {
            service::push_log_with_retry(&self.client, env_config, &better_log, &self.retry_config)
                .await?;
        }
        if self.env_config.verbose {
            eprintln!("{}", better_log);
        }
        Ok(())
    }

    pub async fn debug(&self, log: LogSchema) -> Result<(), LogtailError> {
        let better_log = log.to_betterstack(&self.env_config, LogLevel::Debug);
        println!("{}", better_log);
        Ok(())
    }
}

#[derive(Default)]
pub struct LoggerBuilder {
    app_version: Option<String>,
    verbose: Option<bool>,
    environment: Option<EnvEnum>,
    logs_source_token: Option<String>,
    retry_config: RetryConfig,
}

impl LoggerBuilder {
    pub fn app_version(mut self, app_version: impl Into<String>) -> Self {
        self.app_version = Some(app_version.into());
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = Some(verbose);
        self
    }

    pub fn environment(mut self, environment: EnvEnum) -> Self {
        self.environment = Some(environment);
        self
    }

    pub fn logs_source_token(mut self, token: impl Into<String>) -> Self {
        self.logs_source_token = Some(token.into());
        self
    }

    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.retry_config.max_retries = max_retries;
        self
    }

    pub fn base_delay(mut self, base_delay: Duration) -> Self {
        self.retry_config.base_delay = base_delay;
        self
    }

    pub fn max_delay(mut self, max_delay: Duration) -> Self {
        self.retry_config.max_delay = max_delay;
        self
    }

    pub fn jitter(mut self, jitter: bool) -> Self {
        self.retry_config.jitter = jitter;
        self
    }

    pub fn build(self) -> Logger<ReqwestClient> {
        let env_config = match (self.environment, self.logs_source_token) {
            (Some(environment), Some(token)) => EnvConfig::from_values(
                self.app_version
                    .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string()),
                environment,
                token,
                self.verbose.unwrap_or(true),
            ),
            _ => {
                let version = self
                    .app_version
                    .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
                let verbose = self.verbose.unwrap_or(true);
                EnvConfig::new(version, verbose)
            }
        };

        Logger {
            env_config,
            client: ReqwestClient,
            retry_config: self.retry_config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_client::mock::MockHttpClient;
    use crate::r#struct::env_config::EnvConfig;
    use std::sync::atomic::Ordering;

    fn qa_config() -> EnvConfig {
        EnvConfig::from_values("1.0.0".to_string(), EnvEnum::QA, "token".to_string(), false)
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

    fn no_retry_config() -> RetryConfig {
        RetryConfig {
            max_retries: 0,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(1),
            jitter: false,
        }
    }

    #[tokio::test]
    async fn info_sends_info_level() {
        let mock = MockHttpClient::with_success(None);
        let logger = Logger::with_client(qa_config(), mock);

        let _ = logger.info(test_log()).await;

        let body = logger.client.captured_body.lock().unwrap().clone().unwrap();
        assert_eq!(body["level"], "Info");
    }

    #[tokio::test]
    async fn warn_sends_warn_level() {
        let mock = MockHttpClient::with_success(None);
        let logger = Logger::with_client(qa_config(), mock);

        let _ = logger.warn(test_log()).await;

        let body = logger.client.captured_body.lock().unwrap().clone().unwrap();
        assert_eq!(body["level"], "Warn");
    }

    #[tokio::test]
    async fn error_sends_error_level() {
        let mock = MockHttpClient::with_success(None);
        let logger = Logger::with_client(qa_config(), mock);

        let _ = logger.error(test_log()).await;

        let body = logger.client.captured_body.lock().unwrap().clone().unwrap();
        assert_eq!(body["level"], "Error");
    }

    #[tokio::test]
    async fn debug_skips_http() {
        let mock = MockHttpClient::with_success(None);
        let logger = Logger::with_client(qa_config(), mock);

        let _ = logger.debug(test_log()).await;

        assert_eq!(logger.client.call_count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn local_env_skips_http() {
        let mock = MockHttpClient::with_success(None);
        let logger = Logger::with_client(local_config(), mock);

        let _ = logger.info(test_log()).await;
        let _ = logger.warn(test_log()).await;
        let _ = logger.error(test_log()).await;

        assert_eq!(logger.client.call_count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn non_local_env_sends_http() {
        let mock = MockHttpClient::with_success(None);
        let logger = Logger::with_client(qa_config(), mock);

        let _ = logger.info(test_log()).await;

        assert_eq!(logger.client.call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn info_returns_ok_on_success() {
        let mock = MockHttpClient::with_success(None);
        let logger = Logger::with_client_and_retry(qa_config(), mock, no_retry_config());

        let result = logger.info(test_log()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn info_returns_err_on_failure() {
        let mock = MockHttpClient::with_error("server error");
        let logger = Logger::with_client_and_retry(qa_config(), mock, no_retry_config());

        let result = logger.info(test_log()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn debug_returns_ok() {
        let mock = MockHttpClient::with_success(None);
        let logger = Logger::with_client(qa_config(), mock);

        let result = logger.debug(test_log()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn local_env_returns_ok() {
        let mock = MockHttpClient::with_error("server error");
        let logger = Logger::with_client(local_config(), mock);

        let result = logger.info(test_log()).await;
        assert!(result.is_ok());
    }

    // --- LoggerBuilder tests ---

    #[test]
    fn builder_with_explicit_env_and_token() {
        let logger = Logger::builder()
            .app_version("2.0.0")
            .verbose(false)
            .environment(EnvEnum::Prod)
            .logs_source_token("my-token")
            .build();

        assert_eq!(logger.env_config().app_version, "2.0.0");
        assert!(!logger.env_config().verbose);
        assert_eq!(logger.env_config().environment, EnvEnum::Prod);
        assert_eq!(logger.env_config().logs_source_token, "my-token");
    }

    #[test]
    fn builder_retry_options() {
        let logger = Logger::builder()
            .environment(EnvEnum::QA)
            .logs_source_token("tok")
            .max_retries(5)
            .base_delay(Duration::from_millis(200))
            .max_delay(Duration::from_millis(800))
            .jitter(false)
            .build();

        assert_eq!(logger.retry_config().max_retries, 5);
        assert_eq!(logger.retry_config().base_delay, Duration::from_millis(200));
        assert_eq!(logger.retry_config().max_delay, Duration::from_millis(800));
        assert!(!logger.retry_config().jitter);
    }

    #[test]
    fn builder_defaults_without_overrides() {
        let logger = Logger::builder()
            .environment(EnvEnum::QA)
            .logs_source_token("tok")
            .build();

        assert_eq!(logger.env_config().app_version, env!("CARGO_PKG_VERSION"));
        assert!(logger.env_config().verbose);
        assert_eq!(logger.retry_config().max_retries, 3);
        assert_eq!(logger.retry_config().base_delay, Duration::from_secs(1));
        assert_eq!(logger.retry_config().max_delay, Duration::from_secs(5));
        assert!(logger.retry_config().jitter);
    }

    // --- Logger::with_retry tests ---

    #[tokio::test]
    async fn with_retry_uses_custom_retry_config() {
        let custom = RetryConfig {
            max_retries: 1,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(1),
            jitter: false,
        };
        let mock = MockHttpClient::with_sequence(vec![
            Err((500, "fail".to_string())),
            Ok(Some(serde_json::json!({"ok": true}))),
        ]);
        let logger = Logger::with_client_and_retry(qa_config(), mock, custom);

        let result = logger.info(test_log()).await;
        assert!(result.is_ok());
        // 1 initial + 1 retry = 2 calls
        assert_eq!(logger.client.call_count.load(Ordering::SeqCst), 2);
    }
}
