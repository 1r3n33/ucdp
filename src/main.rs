use actix_web::{post, web, App, Error, HttpResponse, HttpServer};
use isahc::prelude::ReadResponseExt;
use std::collections::HashMap;

mod ucdp;
use ucdp::Client;

struct AppState {
    clients: HashMap<String, Client>,
}

#[post("/{client_id}")]
async fn proxy(
    req_body: String,
    client_id: web::Path<String>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let client = &data.clients[client_id.as_str()];

    let mut result = client.client.post(&client.address, req_body).unwrap();

    Ok(HttpResponse::Ok().body(result.text()?))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = ucdp::Config::new(String::from("config/Main"));

    let state = web::Data::new(AppState {
        clients: config.get_clients(),
    });

    HttpServer::new(move || App::new().app_data(state.clone()).service(proxy))
        .bind(config.get_server_binding_address())?
        .run()
        .await
}
