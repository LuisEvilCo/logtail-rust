use serde::Serialize;
use std::env;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(Debug, EnumString, Display, PartialEq, Serialize, Clone)]
pub enum EnvEnum {
    #[strum(serialize = "local")]
    Local,
    #[strum(serialize = "qa")]
    QA,
    #[strum(serialize = "preprod")]
    PreProd,
    #[strum(serialize = "prod")]
    Prod,
}

pub struct EnvConfig {
    pub app_version: String,
    pub environment: EnvEnum,
    pub logs_source_token: String,
    pub verbose: bool,
}

impl Default for EnvConfig {
    /// Configures environment with the default app version as _this_ cargo package version.
    /// default level for Logger is verbose
    fn default() -> Self {
        Self::new(env!("CARGO_PKG_VERSION").to_string(), true)
    }
}
impl EnvConfig {
    pub fn from_values(
        app_version: String,
        environment: EnvEnum,
        logs_source_token: String,
        verbose: bool,
    ) -> Self {
        EnvConfig {
            app_version,
            environment,
            logs_source_token,
            verbose,
        }
    }

    pub fn new(app_version: String, verbose: bool) -> Self {
        dotenv::dotenv().ok();
        let environment_string = env::var("ENVIRONMENT").expect("missing variable: ENVIRONMENT");
        let environment = EnvEnum::from_str(&environment_string).unwrap();
        let logs_source_token =
            env::var("LOGS_SOURCE_TOKEN").expect("missing variable: LOGS_SOURCE_TOKEN");

        EnvConfig {
            app_version,
            environment,
            logs_source_token,
            verbose,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    // --- EnvEnum tests ---

    #[test]
    fn display_all_variants() {
        assert_eq!(EnvEnum::Local.to_string(), "local");
        assert_eq!(EnvEnum::QA.to_string(), "qa");
        assert_eq!(EnvEnum::PreProd.to_string(), "preprod");
        assert_eq!(EnvEnum::Prod.to_string(), "prod");
    }

    #[test]
    fn parse_valid_variants() {
        assert_eq!(EnvEnum::from_str("local").unwrap(), EnvEnum::Local);
        assert_eq!(EnvEnum::from_str("qa").unwrap(), EnvEnum::QA);
        assert_eq!(EnvEnum::from_str("preprod").unwrap(), EnvEnum::PreProd);
        assert_eq!(EnvEnum::from_str("prod").unwrap(), EnvEnum::Prod);
    }

    #[test]
    fn parse_invalid_returns_err() {
        assert!(EnvEnum::from_str("staging").is_err());
        assert!(EnvEnum::from_str("LOCAL").is_err());
        assert!(EnvEnum::from_str("").is_err());
    }

    #[test]
    fn equality() {
        assert_eq!(EnvEnum::Local, EnvEnum::Local);
        assert_eq!(EnvEnum::Prod, EnvEnum::Prod);
        assert_ne!(EnvEnum::Local, EnvEnum::Prod);
        assert_ne!(EnvEnum::QA, EnvEnum::PreProd);
    }

    #[test]
    fn serde_serialize() {
        assert_eq!(serde_json::to_string(&EnvEnum::Local).unwrap(), "\"Local\"");
        assert_eq!(serde_json::to_string(&EnvEnum::QA).unwrap(), "\"QA\"");
        assert_eq!(
            serde_json::to_string(&EnvEnum::PreProd).unwrap(),
            "\"PreProd\""
        );
        assert_eq!(serde_json::to_string(&EnvEnum::Prod).unwrap(), "\"Prod\"");
    }

    // --- EnvConfig::from_values tests ---

    #[test]
    fn from_values_sets_all_fields() {
        let config = EnvConfig::from_values(
            "1.2.3".to_string(),
            EnvEnum::Prod,
            "my-token".to_string(),
            false,
        );

        assert_eq!(config.app_version, "1.2.3");
        assert_eq!(config.environment, EnvEnum::Prod);
        assert_eq!(config.logs_source_token, "my-token");
        assert!(!config.verbose);
    }

    // --- EnvConfig::new tests (env-var dependent, must run serially) ---

    #[test]
    #[serial]
    fn new_reads_env_vars() {
        env::set_var("ENVIRONMENT", "qa");
        env::set_var("LOGS_SOURCE_TOKEN", "test-token-123");

        let config = EnvConfig::new("0.5.0".to_string(), true);

        assert_eq!(config.app_version, "0.5.0");
        assert_eq!(config.environment, EnvEnum::QA);
        assert_eq!(config.logs_source_token, "test-token-123");
        assert!(config.verbose);
    }

    #[test]
    #[serial]
    fn default_uses_cargo_version() {
        env::set_var("ENVIRONMENT", "local");
        env::set_var("LOGS_SOURCE_TOKEN", "token");

        let config = EnvConfig::default();

        assert_eq!(config.app_version, env!("CARGO_PKG_VERSION"));
        assert!(config.verbose);
    }

    #[test]
    #[serial]
    #[should_panic(expected = "missing variable: ENVIRONMENT")]
    fn new_panics_missing_environment() {
        env::remove_var("ENVIRONMENT");
        env::set_var("LOGS_SOURCE_TOKEN", "token");

        EnvConfig::new("1.0.0".to_string(), false);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn new_panics_invalid_environment() {
        env::set_var("ENVIRONMENT", "staging");
        env::set_var("LOGS_SOURCE_TOKEN", "token");

        EnvConfig::new("1.0.0".to_string(), false);
    }

    #[test]
    #[serial]
    #[should_panic(expected = "missing variable: LOGS_SOURCE_TOKEN")]
    fn new_panics_missing_token() {
        env::set_var("ENVIRONMENT", "local");
        env::remove_var("LOGS_SOURCE_TOKEN");

        EnvConfig::new("1.0.0".to_string(), false);
    }
}
