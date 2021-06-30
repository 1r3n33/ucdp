use actix_web::{post, web, App, Error, HttpResponse, HttpServer};
use futures::executor::block_on;
use std::thread;
use std::time::Duration;

mod ucdp;

struct AppState {
    kafka_stream_producer: Box<dyn ucdp::StreamProducer>,
}

#[post("/")]
async fn proxy(data: String, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    std::thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        block_on(state.kafka_stream_producer.produce(data));
    });

    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = ucdp::Config::new(String::from("config/Main"));

    let state = web::Data::new(AppState {
        kafka_stream_producer: Box::new(config.get_kafka_stream_producer()),
    });

    HttpServer::new(move || App::new().app_data(state.clone()).service(proxy))
        .bind(config.get_server_binding_address())?
        .run()
        .await
}
