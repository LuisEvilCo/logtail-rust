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
