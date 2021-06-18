use actix_web::{post, web, App, Error, HttpResponse, HttpServer};
use isahc::{prelude::*, HttpClient};

struct AppState {
    clients: Vec<isahc::HttpClient>,
}

#[post("/")]
async fn proxy(req_body: String, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut result = data.clients[0]
        .post("http://127.0.0.1:8090", req_body)
        .unwrap();

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
        .map(|_| HttpClient::new().unwrap())
        .collect();

    let state = web::Data::new(AppState { clients: clients });

    HttpServer::new(move || App::new().app_data(state.clone()).service(proxy))
        .bind(bind_config)?
        .run()
        .await
}
