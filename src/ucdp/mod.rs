#[derive(Debug)]
pub struct Client {
    pub id: String,
    pub address: String,
    pub client: isahc::HttpClient,
}

pub struct KafkaStreamProducer {
    pub topic: String,
    pub producer: rdkafka::producer::FutureProducer,
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

    impl PartialEq for Client {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id && self.address == other.address
        }
    }

    fn config() -> Config {
        let mut config = config::Config::default();
        config.set("server.bind", "0.0.0.0:0000").unwrap();
        config.set("clients.first", "1.1.1.1:1111").unwrap();
        config.set("clients.second", "2.2.2.2:2222").unwrap();
        return Config { config: config };
    }

    #[test]
    fn config_get_server_binding_address() {
        assert_eq!(config().get_server_binding_address(), "0.0.0.0:0000");
    }
}
