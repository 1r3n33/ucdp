use crate::ucdp::api::{ErrorResponse, Event, OkResponse};
use crate::ucdp::config::Config;
use crate::ucdp::stream::Events;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{http::header, post, web, App, HttpResponse, HttpServer};
use uuid::Uuid;

struct AppState {
    sender: crossbeam_channel::Sender<Events>,
}

#[post("/v1/events")]
async fn proxy(events: web::Json<Vec<Event>>, state: web::Data<AppState>) -> HttpResponse {
    if events.len() == 0 {
        return HttpResponse::BadRequest().json(&ErrorResponse {
            error: String::from("Events array must not be empty."),
        });
    }
    if events.len() > 100 {
        return HttpResponse::BadRequest().json(&ErrorResponse {
            error: String::from("Events array must not be larger than 100 events."),
        });
    }

    // Create a new token
    let token = Uuid::new_v4().to_hyphenated().to_string();

    // Send events. Do not wait.
    let events = Events {
        token: token.clone(),
        events: events.to_vec(),
    };
    let _ = state.sender.send(events);

    // Respond immediately
    HttpResponse::Ok().json(&OkResponse { token })
}

pub async fn run_http_server(sender: crossbeam_channel::Sender<Events>) -> std::io::Result<()> {
    let config = Config::new(String::from("config/Main"));

    let state = web::Data::new(AppState { sender });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(proxy)
            .wrap(Logger::default())
            .wrap(
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

#[cfg(test)]
mod tests {
    use crate::ucdp::web::Event;
    use crate::ucdp::web::{proxy, AppState, Events};
    use actix_http::http::Method;
    use actix_web::test::{init_service, TestRequest};
    use actix_web::{dev::Service, http::StatusCode, web, App};
    use crossbeam_channel::unbounded;

    #[actix_rt::test]
    async fn http_server_simple_request_ok() {
        let (sender, _) = unbounded::<Events>();
        let state = web::Data::new(AppState { sender });
        let mut service = init_service(App::new().app_data(state.clone()).service(proxy)).await;
        let request = TestRequest::default()
            .uri("/v1/events")
            .method(Method::POST)
            .set_json(&vec![Event {
                name: String::from("event1"),
            }])
            .to_request();
        let response = service.call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
