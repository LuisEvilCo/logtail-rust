use crate::r#struct::env_config::EnvConfig;

mod http_client;
mod r#struct;

struct Logger {
    #[allow(dead_code)]
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
    pub async fn info() {
        todo!()
    }

    #[allow(dead_code)]
    pub async fn warn() {
        todo!()
    }

    #[allow(dead_code)]
    pub async fn error() {
        todo!()
    }

    #[allow(dead_code)]
    pub async fn debug() {
        todo!()
    }
}

pub fn add(left: usize, right: usize) -> usize {
    println!("hi Rodrigo");
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
