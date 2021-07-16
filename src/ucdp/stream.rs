use crate::ucdp::api::Event;
use async_trait::async_trait;
use rdkafka::producer::FutureRecord;
use std::time::Duration;

#[async_trait]
pub trait StreamProducer {
    async fn produce(&self, token: &str, event: &Event);
}

pub struct KafkaStreamProducer {
    pub topic: String,
    pub producer: rdkafka::producer::FutureProducer,
}

#[async_trait]
impl StreamProducer for KafkaStreamProducer {
    async fn produce(&self, token: &str, event: &Event) {
        let _ = self
            .producer
            .send(
                FutureRecord::to(&self.topic)
                    .payload(&serde_json::to_string(&event).unwrap_or_default())
                    .key(token),
                Duration::from_secs(0),
            )
            .await;
    }
}
