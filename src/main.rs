use actix_cors::Cors;
use actix_web::{http::header, post, web, App, Error, HttpServer};
use crossbeam_channel::{select, unbounded};
use futures::executor::block_on;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

mod ucdp;

struct AppState {
    sender: crossbeam_channel::Sender<ucdp::stream::Events>,
}

#[post("/v1/events")]
async fn proxy(
    events: web::Json<Vec<ucdp::api::Event>>,
    state: web::Data<AppState>,
) -> Result<web::Json<ucdp::api::OkResponse>, Error> {
    // Create a new token
    let token = Uuid::new_v4().to_hyphenated().to_string();

    // Send events. Do not wait.
    let events = ucdp::stream::Events {
        token: token.clone(),
        events: events.to_vec(),
    };
    let _ = state.sender.send(events);

    // Respond immediately
    Ok(web::Json(ucdp::api::OkResponse { token }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (sender, receiver) = unbounded::<ucdp::stream::Events>();

    let config = ucdp::config::Config::new(String::from("config/Main"));

    // Start thread that will receive events to send them to the stream
    let config_for_stream_producer_loop = config.clone();
    let stream_producer_loop = async move {
        let stream_producer = config_for_stream_producer_loop.get_stream_producer();
        loop {
            select! {
                recv(receiver) -> res => {
                    if let Ok(events) = res {
                        thread::sleep(Duration::from_secs(3));
                        println!("{}", events.token);
                        stream_producer.produce(&events).await;
                    }
                }
            }
        }
    };
    thread::spawn(|| block_on(stream_producer_loop));

    // Start web service
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
