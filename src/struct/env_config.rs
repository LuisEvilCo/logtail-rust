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
