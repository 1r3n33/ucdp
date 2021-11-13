use crate::config::Config;
use crate::stream::events::Events;
use async_trait::async_trait;
use log::{info, warn};
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::Message;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("config error")]
    Config(#[from] crate::config::Error),

    #[error("unknown connector: {0}")]
    Kafka(#[from] rdkafka::error::KafkaError),

    #[error("deserialization error")]
    Deserialization,
}

#[async_trait]
pub trait StreamConsumer: Send + Sync {
    async fn consume(&self);
}

struct KafkaStreamConsumer {
    pub kafka_consumer: rdkafka::consumer::StreamConsumer,
    pub events_consumer: Box<dyn EventsConsumer>,
}

#[async_trait]
pub trait EventsConsumer: Send + Sync {
    async fn consume(&self, events: &Events);
}

struct DebugEventsConsumer {}

#[async_trait]
impl EventsConsumer for DebugEventsConsumer {
    async fn consume(&self, events: &Events) {
        info!("{:?}", events)
    }
}

#[async_trait]
impl StreamConsumer for KafkaStreamConsumer {
    async fn consume(&self) {
        match self.kafka_consumer.recv().await {
            Err(_) => {}
            Ok(message) => {
                match message.payload_view::<[u8]>() {
                    Some(Ok(payload)) => match serde_json::from_slice::<Events>(payload) {
                        Ok(events) => self.events_consumer.consume(&events).await,
                        Err(error) => {
                            warn!("Error while deserializing message payload: {:?}", error)
                        }
                    },
                    Some(Err(error)) => {
                        warn!("Error while fetching message payload: {:?}", error);
                    }
                    None => {
                        warn!("Error while fetching message payload: Unknown error");
                    }
                }

                self.kafka_consumer
                    .commit_message(&message, CommitMode::Async)
                    .unwrap();
            }
        }
    }
}

pub struct StreamConsumerBuilder {}

impl StreamConsumerBuilder {
    pub fn build(config: &Config) -> Result<Box<dyn StreamConsumer>, Error> {
        let kafka_broker = config
            .get_str("stream.kafka.broker")
            .map_err(Error::Config)?;
        let kafka_topic = config
            .get_str("stream.kafka.topic")
            .map_err(Error::Config)?;

        let kafka_consumer: rdkafka::consumer::StreamConsumer =
            rdkafka::config::ClientConfig::new()
                .set("group.id", "workers")
                .set("bootstrap.servers", kafka_broker)
                .create()
                .map_err(Error::Kafka)?;

        kafka_consumer
            .subscribe(&[kafka_topic.as_str()])
            .map_err(Error::Kafka)?;

        let stream_consumer = KafkaStreamConsumer {
            kafka_consumer,
            events_consumer: Box::new(DebugEventsConsumer {}),
        };

        Ok(Box::new(stream_consumer))
    }
}

#[cfg(test)]
mod tests {
    use crate::stream::consumer::{Config, StreamConsumerBuilder};

    #[test]
    fn stream_consumer_builder_ok() {
        let mut config = config::Config::default();
        let _ = config.set("stream.kafka.topic", "topic");
        let _ = config.set("stream.kafka.broker", "0.0.0.0:0000");
        let config = Config::from(config);

        let res = StreamConsumerBuilder::build(&config);
        assert!(res.is_ok());
    }

    #[test]
    fn stream_consumer_builder_err() {
        let config = config::Config::default();
        let config = Config::from(config);

        let res = StreamConsumerBuilder::build(&config);
        assert!(res.is_err());
    }
}
