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
