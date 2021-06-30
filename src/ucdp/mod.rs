use async_trait::async_trait;
use rdkafka::producer::FutureRecord;
use std::time::Duration;

#[async_trait]
pub trait StreamProducer: Send + Sync {
    async fn produce(&self, data: String);
}

pub struct KafkaStreamProducer {
    pub topic: String,
    pub producer: rdkafka::producer::FutureProducer,
}

#[async_trait]
impl StreamProducer for KafkaStreamProducer {
    async fn produce(&self, data: String) {
        let _ = self
            .producer
            .send(
                FutureRecord::to(&self.topic)
                    .payload(&data)
                    .key(&String::from("key")),
                Duration::from_secs(0),
            )
            .await;
    }
}

pub struct Config {
    config: config::Config,
}

impl Config {
    pub fn new(path: String) -> Self {
        let mut config = config::Config::default();
        config.merge(config::File::with_name(&path)).unwrap();
        return Config { config: config };
    }

    pub fn get_server_binding_address(&self) -> String {
        return self.config.get_str("server.bind").unwrap();
    }

    pub fn get_kafka_stream_producer(&self) -> KafkaStreamProducer {
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
        config.set("server.bind", "0.0.0.0:0000").unwrap();
        config.set("stream.kafka.broker", "1.1.1.1:1111").unwrap();
        config.set("stream.kafka.topic", "kafka_topic").unwrap();
        return Config { config: config };
    }

    #[test]
    fn config_get_server_binding_address() {
        assert_eq!(config().get_server_binding_address(), "0.0.0.0:0000");
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
}
