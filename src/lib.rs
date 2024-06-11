use crate::r#struct::env_config::EnvConfig;
use crate::r#struct::log_level::LogLevel;
use crate::r#struct::log_schema::LogSchema;

mod http_client;
mod r#struct;

struct Logger {
    env_config: EnvConfig,
}

impl Default for Logger {
    fn default() -> Self {
        let env_config = EnvConfig::default();
        Logger { env_config }
    }
}

impl Logger {
    #[allow(dead_code)]
    pub async fn info(&self, log: LogSchema) {
        let env_config = &self.env_config;
        let better_log = log.to_betterstack(env_config, LogLevel::Info);
        // TODO: add api call
        if env_config.verbose {
            println!("{}", better_log);
        }
    }

    #[allow(dead_code)]
    pub async fn warn(&self, log: LogSchema) {
        let better_log = log.to_betterstack(&self.env_config, LogLevel::Warn);
        // TODO : add api call
        if self.env_config.verbose {
            println!("{}", better_log);
        }
    }

    #[allow(dead_code)]
    pub async fn error(&self, log: LogSchema) {
        let better_log = log.to_betterstack(&self.env_config, LogLevel::Error);
        // TODO : add api call
        if self.env_config.verbose {
            eprintln!("{}", better_log);
        }
    }

    #[allow(dead_code)]
    pub async fn debug(&self, log: LogSchema) {
        let better_log = log.to_betterstack(&self.env_config, LogLevel::Debug);
        println!("{}", better_log);
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
