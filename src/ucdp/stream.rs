use crate::ucdp::api::Event;
use crate::ucdp::config::Config;
use async_trait::async_trait;
use crossbeam_channel::select;
use futures::executor::block_on;
use rdkafka::producer::FutureRecord;
use serde::Serialize;
use std::thread;
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
    let stream_producer = config.get_stream_producer();
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

#[cfg(test)]
mod tests {
    use super::{async_trait, block_on, stream_producer_loop, Events, StreamProducer};
    use crossbeam_channel::{unbounded, RecvError};
    use std::fmt;

    impl fmt::Debug for Events {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self.token)
        }
    }

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
}
