use actix_cors::Cors;
use actix_web::{http::header, post, web, App, Error, HttpServer};
use crossbeam_channel::{select, unbounded};
use futures::executor::block_on;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

mod ucdp;

struct AppState {
    sender: crossbeam_channel::Sender<ucdp::api::Event>,
}

#[post("/v1/events")]
async fn proxy(
    events: web::Json<Vec<ucdp::api::Event>>,
    state: web::Data<AppState>,
) -> Result<web::Json<ucdp::api::OkResponse>, Error> {
    // Create a new token
    let token = Uuid::new_v4().to_hyphenated().to_string();

    let _ = state.sender.send(events[0].clone());

    Ok(web::Json(ucdp::api::OkResponse { token }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (sender, receiver) = unbounded::<ucdp::api::Event>();

    let config = ucdp::config::Config::new(String::from("config/Main"));

    let config_for_stream_producer_loop = config.clone();
    let stream_producer_loop = async move {
        let stream_producer = config_for_stream_producer_loop.get_stream_producer();
        loop {
            select! {
                recv(receiver) -> res => {
                    if let Ok(event) = res {
                        thread::sleep(Duration::from_secs(3));
                        println!("{}", event.name);
                        stream_producer.produce("token: &str", &event).await;
                    }
                }
            }
        }
    };
    thread::spawn(|| block_on(stream_producer_loop));

    let state = web::Data::new(AppState { sender });
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
