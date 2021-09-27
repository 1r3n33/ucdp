use crate::ucdp::stream::{KafkaStreamProducer, StreamProducer};
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

    fn get_kafka_stream_producer(&self) -> KafkaStreamProducer {
        KafkaStreamProducer {
            topic: self.config.get_str("stream.kafka.topic").unwrap(),
            producer: rdkafka::config::ClientConfig::new()
                .set(
                    "bootstrap.servers",
                    self.config.get_str("stream.kafka.broker").unwrap(),
                )
                .create()
                .expect("Kafka producer creation error"),
        }
    }

    pub fn get_stream_producer(&self) -> Box<dyn StreamProducer> {
        Box::new(self.get_kafka_stream_producer())
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
    use std::fmt;

    impl PartialEq for KafkaStreamProducer {
        fn eq(&self, other: &Self) -> bool {
            self.topic == other.topic
        }
    }

    impl fmt::Debug for KafkaStreamProducer {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self.topic)
        }
    }

    fn config() -> Config {
        let mut config = config::Config::default();
        config.set("stream.kafka.broker", "1.1.1.1:1111").unwrap();
        config.set("stream.kafka.topic", "kafka_topic").unwrap();
        Config { config }
    }

    #[test]
    fn config_get_kafka_stream_producer() {
        assert_eq!(
            config().get_kafka_stream_producer(),
            KafkaStreamProducer {
                topic: String::from("kafka_topic"),
                producer: rdkafka::config::ClientConfig::new().create().unwrap(),
            }
        )
    }

    #[test]
    fn config_get_str() {
        let mut config = config::Config::default();
        let _ = config.set("abc", "123");

        let config = Config { config };
        assert_eq!(config.get_str("abc").unwrap().as_str(), "123");
        assert!(config.get_str("def").is_err());
    }
}
