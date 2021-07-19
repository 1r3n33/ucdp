use actix_cors::Cors;
use actix_web::{http::header, post, web, App, Error, HttpServer};
use crossbeam_channel::unbounded;
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
    ucdp::stream::spawn_stream_producer_thread(receiver);

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
