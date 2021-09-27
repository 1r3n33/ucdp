use config::{ConfigError, Environment};
use thiserror::Error;

#[derive(Clone)]
pub struct Config {
    config: config::Config,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("config error")]
    Config(#[from] ConfigError),
}

impl Config {
    pub fn new(path: String) -> Self {
        let mut config = config::Config::default();
        let _ = config.merge(config::File::with_name(&path));
        let _ = config.merge(
            Environment::with_prefix("ucdp")
                .separator("_")
                .ignore_empty(false),
        );

        Config { config }
    }

    pub fn get_str(&self, key: &str) -> Result<String, Error> {
        self.config.get_str(key).map_err(Error::Config)
    }
}

#[cfg(test)]
impl Config {
    pub(in crate) fn from(config: config::Config) -> Self {
        Config { config }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_get_str() {
        let mut config = config::Config::default();
        let _ = config.set("abc", "123");

        let config = Config { config };
        assert_eq!(config.get_str("abc").unwrap().as_str(), "123");
        assert!(config.get_str("def").is_err());
    }
}
