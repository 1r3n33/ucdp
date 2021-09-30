use ucdp::config::Config;
use ucdp::stream::consumer::{Error, StreamConsumerBuilder};

#[actix_web::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let config = Config::new(String::from("config/Main"));

    let stream_consumer = StreamConsumerBuilder::build(&config)?;
    loop {
        stream_consumer.consume().await;
    }
}
