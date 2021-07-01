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
