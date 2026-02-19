use serde::Serialize;
use strum_macros::{Display, EnumString};

#[derive(Debug, EnumString, Display, PartialEq, Serialize, Clone)]
pub enum LogLevel {
    #[strum(serialize = "info")]
    Info,
    #[strum(serialize = "warn")]
    Warn,
    #[strum(serialize = "error")]
    Error,
    #[strum(serialize = "debug")]
    Debug,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn display_all_variants() {
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Warn.to_string(), "warn");
        assert_eq!(LogLevel::Error.to_string(), "error");
        assert_eq!(LogLevel::Debug.to_string(), "debug");
    }

    #[test]
    fn parse_valid_variants() {
        assert_eq!(LogLevel::from_str("info").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("warn").unwrap(), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("error").unwrap(), LogLevel::Error);
        assert_eq!(LogLevel::from_str("debug").unwrap(), LogLevel::Debug);
    }

    #[test]
    fn parse_invalid_returns_err() {
        assert!(LogLevel::from_str("unknown").is_err());
        assert!(LogLevel::from_str("INFO").is_err());
        assert!(LogLevel::from_str("").is_err());
    }

    #[test]
    fn equality() {
        assert_eq!(LogLevel::Info, LogLevel::Info);
        assert_ne!(LogLevel::Info, LogLevel::Warn);
        assert_ne!(LogLevel::Error, LogLevel::Debug);
    }

    #[test]
    fn serde_json_serialize() {
        assert_eq!(serde_json::to_string(&LogLevel::Info).unwrap(), "\"Info\"");
        assert_eq!(serde_json::to_string(&LogLevel::Warn).unwrap(), "\"Warn\"");
        assert_eq!(
            serde_json::to_string(&LogLevel::Error).unwrap(),
            "\"Error\""
        );
        assert_eq!(
            serde_json::to_string(&LogLevel::Debug).unwrap(),
            "\"Debug\""
        );
    }
}
