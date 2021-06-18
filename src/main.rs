use actix_web::{post, web, App, Error, HttpResponse, HttpServer};
use isahc::{prelude::*, HttpClient};

struct AppState {
    client: isahc::HttpClient,
}

#[post("/")]
async fn proxy(req_body: String, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut result = data.client.post("http://127.0.0.1:8090", req_body).unwrap();

    Ok(HttpResponse::Ok().body(result.text()?))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut config = config::Config::default();
    config
        .merge(config::File::with_name("config/Main"))
        .unwrap();

    let bind_config = config.get::<String>("server.bind").unwrap();

    HttpServer::new(|| {
        App::new()
            .data(AppState {
                client: HttpClient::new().unwrap(),
            })
            .service(proxy)
    })
    .bind(bind_config)?
    .run()
    .await
}
