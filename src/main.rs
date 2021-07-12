use actix_cors::Cors;
use actix_web::{http::header, post, web, App, Error, HttpServer};
use futures::executor::block_on;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

mod ucdp;

struct AppState {
    stream_producer: Box<dyn ucdp::stream::StreamProducer>,
}

#[post("/v1/events")]
async fn proxy(
    events: web::Json<Vec<ucdp::api::Event>>,
    state: web::Data<AppState>,
) -> Result<web::Json<ucdp::api::OkResponse>, Error> {
    // Create a new token
    let token = Uuid::new_v4().to_hyphenated().to_string();

    // Post events to stream
    let token_for_closure = token.clone();
    std::thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        block_on(
            state
                .stream_producer
                .produce(&token_for_closure, &events[0]),
        );
    });

    Ok(web::Json(ucdp::api::OkResponse { token }))
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
