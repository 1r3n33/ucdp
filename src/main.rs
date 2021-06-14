use actix_web::{client, post, web, App, Error, HttpResponse, HttpServer};

struct AppState {
    client: client::Client,
}

#[post("/")]
async fn proxy(req_body: String, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut result = data
        .client
        .post("http://127.0.0.1:8090")
        .send_body(req_body)
        .await?;

    Ok(HttpResponse::Ok().body(result.body().await?))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("config/Main"))
        .unwrap();

    let bind = settings.get::<String>("server.bind").unwrap();

    HttpServer::new(|| {
        App::new()
            .data(AppState {
                client: client::Client::default(),
            })
            .service(proxy)
    })
    .bind(bind)?
    .run()
    .await
}
