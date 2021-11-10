use crate::config::Config;
use crate::stream::events::Events;
use async_trait::async_trait;
use crossbeam_channel::select;
use futures::executor::block_on;
use rdkafka::producer::FutureRecord;
use std::thread;
use std::time::Duration;

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
        thread::sleep(Duration::from_secs(3));
        println!("{}", events.token);

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

pub fn spawn_stream_producer_thread(receiver: crossbeam_channel::Receiver<Events>) {
    let config = Config::new(String::from("config/Main"));
    let stream_producer = StreamProducerBuilder::build(&config).unwrap();
    thread::spawn(|| block_on(stream_producer_loop(stream_producer, receiver)));
}

async fn stream_producer_loop(
    stream_producer: Box<dyn StreamProducer>,
    receiver: crossbeam_channel::Receiver<Events>,
) {
    loop {
        select! {
            recv(receiver) -> res => match res {
                Ok(events) => {
                    stream_producer.produce(&events).await
                }
                Err(_) => return
            }
        }
    }
}

#[derive(Debug)]
pub struct Error {}

pub struct StreamProducerBuilder {}

impl StreamProducerBuilder {
    pub fn build(config: &Config) -> Result<Box<dyn StreamProducer>, Error> {
        let stream_producer = KafkaStreamProducer {
            topic: config.get_str("stream.kafka.topic").map_err(|_| Error {})?,
            producer: rdkafka::config::ClientConfig::new()
                .set(
                    "bootstrap.servers",
                    config
                        .get_str("stream.kafka.broker")
                        .map_err(|_| Error {})?,
                )
                .create()
                .map_err(|_| Error {})?,
        };

        Ok(Box::new(stream_producer))
    }
}

#[cfg(test)]
mod tests {
    use super::{async_trait, block_on, stream_producer_loop};
    use crate::config::Config;
    use crate::stream::events::Events;
    use crate::stream::producer::{StreamProducer, StreamProducerBuilder};
    use crossbeam_channel::{unbounded, RecvError};

    impl PartialEq for Events {
        fn eq(&self, other: &Self) -> bool {
            self.token == other.token
        }
    }

    struct TestStreamProducer {}

    #[async_trait]
    impl StreamProducer for TestStreamProducer {
        async fn produce(&self, _: &Events) {}
    }

    #[test]
    fn stream_producer_loop_receive_and_leave() {
        let (sender, receiver) = unbounded::<Events>();

        let stream_producer = TestStreamProducer {};

        let tokens = vec!["token1", "token2", "token3"];
        for token in tokens {
            sender
                .send(Events {
                    token: String::from(token),
                    events: vec![],
                })
                .unwrap();
        }
        drop(sender);

        block_on(stream_producer_loop(
            Box::new(stream_producer),
            receiver.clone(),
        ));

        assert_eq!(receiver.recv(), Err(RecvError))
    }

    #[test]
    fn stream_producer_builder_ok() {
        let mut config = config::Config::default();
        let _ = config.set("stream.kafka.topic", "topic");
        let _ = config.set("stream.kafka.broker", "0.0.0.0:0000");
        let config = Config::from(config);

        let res = StreamProducerBuilder::build(&config);
        assert!(res.is_ok());
    }

    #[test]
    fn stream_producer_builder_err() {
        let config = config::Config::default();
        let config = Config::from(config);

        let res = StreamProducerBuilder::build(&config);
        assert!(res.is_err());
    }
}
