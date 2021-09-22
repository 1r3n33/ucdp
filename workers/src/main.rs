use crate::stream::consumer::StreamConsumerBuilder;

mod stream;

#[derive(Debug)]
struct Error {}

#[actix_web::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let stream_consumer = StreamConsumerBuilder::build().map_err(|_| Error {})?;
    loop {
        stream_consumer.consume().await;
    }
}
