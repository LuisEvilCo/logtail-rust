use serde::Serialize;
use std::fmt::{Display, Formatter};

use crate::r#struct::env_config::{EnvConfig, EnvEnum};
use crate::r#struct::log_level::LogLevel;

#[derive(Debug, Serialize, Clone)]
pub struct BetterStackLogSchema {
    pub env: EnvEnum,
    pub message: String,
    pub context: String,
    pub level: LogLevel,
    pub app_version: String,
}

impl Display for BetterStackLogSchema {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let self_json = serde_json::to_string(self).unwrap();
        write!(f, "{}", self_json)
    }
}

impl BetterStackLogSchema {
    pub(crate) fn new(
        env_config: &EnvConfig,
        level: LogLevel,
        message: String,
        context: String,
    ) -> Self {
        Self {
            env: env_config.environment.clone(),
            message,
            context,
            level,
            app_version: env_config.app_version.clone(),
        }
    }
}
