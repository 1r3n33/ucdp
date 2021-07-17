use crate::ucdp::api::Event;
use async_trait::async_trait;
use rdkafka::producer::FutureRecord;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
pub struct Events {
    pub token: String,
    pub events: Vec<Event>,
}

#[async_trait]
pub trait StreamProducer: Send + Sync {
    async fn produce(&self, events: &Events);
}

pub struct KafkaStreamProducer {
    pub topic: String,
    pub producer: rdkafka::producer::FutureProducer,
}

#[async_trait]
impl StreamProducer for KafkaStreamProducer {
    async fn produce(&self, events: &Events) {
        let _ = self
            .producer
            .send(
                FutureRecord::to(&self.topic)
                    .payload(&serde_json::to_string(&events).unwrap_or_default())
                    .key(&events.token),
                Duration::from_secs(0),
            )
            .await;
    }
}
