use actix_cors::Cors;
use actix_web::{http::header, post, web, App, Error, HttpResponse, HttpServer};
use futures::executor::block_on;
use serde::Deserialize;
use std::thread;
use std::time::Duration;

mod ucdp;

struct AppState {
    stream_producer: Box<dyn ucdp::stream::StreamProducer>,
}

#[derive(Deserialize)]
struct Event {
    name: String,
}

#[post("/v1/events")]
async fn proxy(
    events: web::Json<Vec<Event>>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    std::thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        block_on(state.stream_producer.produce(events[0].name.clone()));
    });

    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = ucdp::config::Config::new(String::from("config/Main"));

    let state = web::Data::new(AppState {
        stream_producer: Box::new(config.get_stream_producer()),
    });

    HttpServer::new(move || {
        App::new().app_data(state.clone()).service(proxy).wrap(
            Cors::default()
                .allow_any_origin()
                .allowed_methods(vec!["POST"])
                .allowed_headers(vec![header::ACCEPT, header::CONTENT_TYPE])
                .max_age(3600),
        )
    })
    .bind(config.get_server_binding_address())?
    .run()
    .await
}
