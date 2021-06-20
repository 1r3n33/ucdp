use actix_web::{post, web, App, Error, HttpResponse, HttpServer};
use isahc::{prelude::*, HttpClient};
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
    let mut config = config::Config::default();
    config
        .merge(config::File::with_name("config/Main"))
        .unwrap();

    let bind_config = config.get_str("server.bind").unwrap();

    let clients_config = config.get_table("clients").unwrap();
    let clients = clients_config
        .into_iter()
        .map(|(client_id, address)| {
            (
                client_id.clone(),
                Client {
                    id: client_id.clone(),
                    address: address.into_str().unwrap(),
                    client: HttpClient::new().unwrap(),
                },
            )
        })
        .collect();

    let state = web::Data::new(AppState { clients: clients });

    HttpServer::new(move || App::new().app_data(state.clone()).service(proxy))
        .bind(bind_config)?
        .run()
        .await
}
