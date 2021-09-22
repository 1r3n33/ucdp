use async_trait::async_trait;
use log::{info, warn};
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::{Headers, Message};

pub struct Error {}

#[async_trait]
pub trait StreamConsumer: Send + Sync {
    async fn consume(&self);
}

struct KafkaStreamConsumer {
    pub kafka_consumer: rdkafka::consumer::StreamConsumer,
}

#[async_trait]
impl StreamConsumer for KafkaStreamConsumer {
    async fn consume(&self) {
        match self.kafka_consumer.recv().await {
            Err(_) => {}
            Ok(message) => {
                let payload = match message.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        warn!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                  message.key(), payload, message.topic(), message.partition(), message.offset(), message.timestamp());
                if let Some(headers) = message.headers() {
                    for i in 0..headers.count() {
                        let header = headers.get(i).unwrap();
                        info!("  Header {:#?}: {:?}", header.0, header.1);
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
    pub fn build() -> Result<Box<dyn StreamConsumer>, Error> {
        let kafka_consumer: rdkafka::consumer::StreamConsumer =
            rdkafka::config::ClientConfig::new()
                .set("group.id", "workers")
                .set("bootstrap.servers", "127.0.0.1:9092")
                .create()
                .map_err(|_| Error {})?;

        kafka_consumer
            .subscribe(&["events"])
            .map_err(|_| Error {})?;

        let stream_consumer = KafkaStreamConsumer { kafka_consumer };

        Ok(Box::new(stream_consumer))
    }
}
