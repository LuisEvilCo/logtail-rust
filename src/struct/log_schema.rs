use crate::r#struct::betterstack_log_schema::BetterStackLogSchema;
use crate::r#struct::env_config::EnvConfig;
use crate::r#struct::log_level::LogLevel;

#[derive(Debug)]
pub struct LogSchema {
    pub message: String,
    pub context: String,
}

impl LogSchema {
    pub fn to_betterstack(&self, env_config: &EnvConfig, level: LogLevel) -> BetterStackLogSchema {
        BetterStackLogSchema::new(
            env_config,
            level,
            self.message.clone(),
            self.context.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r#struct::env_config::{EnvConfig, EnvEnum};

    fn test_config() -> EnvConfig {
        EnvConfig {
            app_version: "0.1.0".to_string(),
            environment: EnvEnum::QA,
            logs_source_token: "test-token".to_string(),
            verbose: false,
        }
    }

    #[test]
    fn to_betterstack_maps_message() {
        let log = LogSchema {
            message: "hello world".to_string(),
            context: "ctx".to_string(),
        };
        let result = log.to_betterstack(&test_config(), LogLevel::Info);
        assert_eq!(result.message, "hello world");
    }

    #[test]
    fn to_betterstack_maps_context() {
        let log = LogSchema {
            message: "msg".to_string(),
            context: "file.rs:42".to_string(),
        };
        let result = log.to_betterstack(&test_config(), LogLevel::Info);
        assert_eq!(result.context, "file.rs:42");
    }

    #[test]
    fn to_betterstack_maps_level() {
        let log = LogSchema {
            message: "msg".to_string(),
            context: "ctx".to_string(),
        };

        assert_eq!(
            log.to_betterstack(&test_config(), LogLevel::Info).level,
            LogLevel::Info
        );
        assert_eq!(
            log.to_betterstack(&test_config(), LogLevel::Warn).level,
            LogLevel::Warn
        );
        assert_eq!(
            log.to_betterstack(&test_config(), LogLevel::Error).level,
            LogLevel::Error
        );
        assert_eq!(
            log.to_betterstack(&test_config(), LogLevel::Debug).level,
            LogLevel::Debug
        );
    }

    #[test]
    fn to_betterstack_maps_env() {
        let mut config = test_config();
        config.environment = EnvEnum::Prod;

        let log = LogSchema {
            message: "msg".to_string(),
            context: "ctx".to_string(),
        };
        let result = log.to_betterstack(&config, LogLevel::Info);
        assert_eq!(result.env, EnvEnum::Prod);
    }

    #[test]
    fn to_betterstack_maps_version() {
        let mut config = test_config();
        config.app_version = "3.2.1".to_string();

        let log = LogSchema {
            message: "msg".to_string(),
            context: "ctx".to_string(),
        };
        let result = log.to_betterstack(&config, LogLevel::Info);
        assert_eq!(result.app_version, "3.2.1");
    }
}
