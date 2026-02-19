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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_schema() -> BetterStackLogSchema {
        BetterStackLogSchema {
            env: EnvEnum::QA,
            message: "test message".to_string(),
            context: "test context".to_string(),
            level: LogLevel::Info,
            app_version: "1.0.0".to_string(),
        }
    }

    #[test]
    fn display_outputs_valid_json() {
        let schema = sample_schema();
        let output = schema.to_string();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn display_contains_all_fields() {
        let schema = sample_schema();
        let output = schema.to_string();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let obj = parsed.as_object().unwrap();

        assert!(obj.contains_key("env"));
        assert!(obj.contains_key("message"));
        assert!(obj.contains_key("context"));
        assert!(obj.contains_key("level"));
        assert!(obj.contains_key("app_version"));
    }

    #[test]
    fn display_field_values_match() {
        let schema = sample_schema();
        let output = schema.to_string();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["message"], "test message");
        assert_eq!(parsed["context"], "test context");
        assert_eq!(parsed["app_version"], "1.0.0");
        assert_eq!(parsed["env"], "QA");
        assert_eq!(parsed["level"], "Info");
    }

    #[test]
    fn clone_produces_equal_fields() {
        let schema = sample_schema();
        let cloned = schema.clone();

        assert_eq!(schema.env, cloned.env);
        assert_eq!(schema.message, cloned.message);
        assert_eq!(schema.context, cloned.context);
        assert_eq!(schema.level, cloned.level);
        assert_eq!(schema.app_version, cloned.app_version);
    }

    #[test]
    fn serde_serialize_contains_expected_keys() {
        let schema = sample_schema();
        let value = serde_json::to_value(&schema).unwrap();
        let obj = value.as_object().unwrap();

        assert_eq!(obj.len(), 5);
        assert_eq!(obj["message"], "test message");
        assert_eq!(obj["context"], "test context");
        assert_eq!(obj["level"], "Info");
        assert_eq!(obj["env"], "QA");
        assert_eq!(obj["app_version"], "1.0.0");
    }

    #[test]
    fn new_maps_fields_from_env_config() {
        let config = EnvConfig {
            app_version: "2.5.0".to_string(),
            environment: EnvEnum::Prod,
            logs_source_token: "token".to_string(),
            verbose: false,
        };

        let schema = BetterStackLogSchema::new(
            &config,
            LogLevel::Error,
            "err msg".to_string(),
            "err ctx".to_string(),
        );

        assert_eq!(schema.env, EnvEnum::Prod);
        assert_eq!(schema.message, "err msg");
        assert_eq!(schema.context, "err ctx");
        assert_eq!(schema.level, LogLevel::Error);
        assert_eq!(schema.app_version, "2.5.0");
    }
}
