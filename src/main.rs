use actix_web::{post, web, App, Error, HttpResponse, HttpServer};
use rdkafka::producer::FutureRecord;
use std::time::Duration;

mod ucdp;
use ucdp::KafkaStreamProducer;

struct AppState {
    kafka_stream_producer: KafkaStreamProducer,
}

#[post("/")]
async fn proxy(req_body: String, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let _ = data
        .kafka_stream_producer
        .producer
        .send(
            FutureRecord::to(&data.kafka_stream_producer.topic)
                .payload(&req_body)
                .key(&String::from("key")),
            Duration::from_secs(0),
        )
        .await;

    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = ucdp::Config::new(String::from("config/Main"));

    let state = web::Data::new(AppState {
        kafka_stream_producer: config.get_kafka_stream_producer(),
    });

    HttpServer::new(move || App::new().app_data(state.clone()).service(proxy))
        .bind(config.get_server_binding_address())?
        .run()
        .await
}
