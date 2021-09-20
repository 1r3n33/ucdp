use crate::ucdp::api::{ErrorResponse, OkResponse};
use crate::ucdp::config::Config;
use crate::ucdp::partners::{Partners, PartnersBuilder};
use actix_cors::Cors;
use actix_web::{http::header, middleware::Logger, post, web, App, HttpResponse, HttpServer};
use uuid::Uuid;

struct AppState {
    sender: crossbeam_channel::Sender<crate::ucdp::stream::Events>,
    partners: Partners,
}

// TODO move to api
#[post("/v1/events")]
async fn proxy(
    req: web::Json<crate::ucdp::api::Events>,
    state: web::Data<AppState>,
) -> HttpResponse {
    if req.events.is_empty() {
        return HttpResponse::BadRequest().json(&ErrorResponse {
            error: String::from("Events array must not be empty."),
        });
    }
    if req.events.len() > 100 {
        return HttpResponse::BadRequest().json(&ErrorResponse {
            error: String::from("Events array must not be larger than 100 events."),
        });
    }

    match state.partners.get_partner(req.partner.as_str()).await {
        Ok(partner) if !partner.enabled => {
            return HttpResponse::Forbidden().json(&ErrorResponse {
                error: String::from("Partner must be enabled."),
            });
        }
        Err(_) => {
            return HttpResponse::Forbidden().json(&ErrorResponse {
                error: String::from("Partner must be set."),
            });
        }
        _ => {}
    }

    // Create a new token
    let token = Uuid::new_v4().to_hyphenated().to_string();

    // Send events. Do not wait.
    let events = crate::ucdp::stream::Events {
        token: token.clone(),
        events: req.events.to_vec(),
    };
    let _ = state.sender.send(events);

    // Respond immediately
    HttpResponse::Ok().json(&OkResponse { token })
}

pub async fn run_http_server(
    sender: crossbeam_channel::Sender<crate::ucdp::stream::Events>,
) -> std::io::Result<()> {
    let config = Config::new(String::from("config/Main"));

    let state = web::Data::new(AppState {
        sender,
        partners: PartnersBuilder::build(&config).unwrap(),
    });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["POST"])
                    .allowed_headers(vec![header::ACCEPT, header::CONTENT_TYPE])
                    .max_age(3600),
            )
            .wrap(Logger::default())
            .service(proxy)
    })
    .bind(config.get_server_binding_address())?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use crate::ucdp::partners::{Error, Partner, PartnersBuilderForTest, PartnersDAO};
    use crate::ucdp::web::{proxy, AppState};
    use actix_http::http::Method;
    use actix_web::test::{init_service, TestRequest};
    use actix_web::{dev::Service, dev::ServiceResponse, http::StatusCode, web, App};
    use async_trait::async_trait;
    use crossbeam_channel::unbounded;

    struct PartnerOptionDAO {
        partner: Option<Partner>,
    }

    #[async_trait]
    impl PartnersDAO for PartnerOptionDAO {
        async fn get_partner(&self, p: &str) -> Result<Partner, Error> {
            self.partner
                .clone()
                .ok_or_else(|| Error::PartnerNotFound(p.to_string()))
        }
    }

    async fn get_response(partner: Option<Partner>) -> ServiceResponse {
        let (sender, _) = unbounded::<crate::ucdp::stream::Events>();
        let state = web::Data::new(AppState {
            sender,
            partners: PartnersBuilderForTest::build(Box::new(PartnerOptionDAO { partner })),
        });
        let service = init_service(App::new().app_data(state.clone()).service(proxy)).await;
        let request = TestRequest::default()
            .uri("/v1/events")
            .method(Method::POST)
            .set_json(&crate::ucdp::api::Events {
                partner: "0x0123456789".into(),
                events: vec![crate::ucdp::api::Event {
                    name: String::from("event1"),
                }],
            })
            .to_request();
        let response = service.call(request).await.unwrap();
        response
    }

    #[actix_rt::test]
    async fn http_server_simple_request_ok() {
        let response = get_response(Some(Partner {
            name: "".into(),
            enabled: true,
        }))
        .await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn http_server_simple_request_err_no_partner() {
        let response = get_response(None).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[actix_rt::test]
    async fn http_server_simple_request_err_partner_disabled() {
        let response = get_response(Some(Partner {
            name: "".into(),
            enabled: false,
        }))
        .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
