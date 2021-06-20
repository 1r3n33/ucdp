use actix_web::{post, web, App, Error, HttpResponse, HttpServer};
use isahc::{prelude::*, HttpClient};

mod ucdp;
use crate::ucdp::Client;

struct AppState {
    clients: Vec<Client>,
}

#[post("/")]
async fn proxy(req_body: String, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let client = &data.clients[0];

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
        .map(|(name, address)| Client {
            name: name,
            address: address.into_str().unwrap(),
            client: HttpClient::new().unwrap(),
        })
        .collect();

    let state = web::Data::new(AppState { clients: clients });

    HttpServer::new(move || App::new().app_data(state.clone()).service(proxy))
        .bind(bind_config)?
        .run()
        .await
}
