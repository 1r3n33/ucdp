use crate::ucdp::api::{ErrorResponse, OkResponse};
use crate::ucdp::dal::{
    AuthorizedPartnersByUserBuilder, AuthorizedPartnersByUserDao, PartnersBuilder, PartnersDao,
};
use actix_cors::Cors;
use actix_web::{http::header, middleware::Logger, post, web, App, HttpResponse, HttpServer};
use ucdp::config::Config;
use uuid::Uuid;

struct AppState {
    sender: crossbeam_channel::Sender<crate::ucdp::stream::Events>,
    partners: Box<dyn PartnersDao>,
    authorized_partners_by_user: Box<dyn AuthorizedPartnersByUserDao>,
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
    // Check partner id
    let partner_id = req.partner.as_str();
    match state.partners.get_partner(partner_id).await {
        Ok(partner) if !partner.enabled => {
            return HttpResponse::Forbidden().json(&ErrorResponse {
                error: String::from("Partner must be enabled."),
            });
        }
        Err(error) => {
            return HttpResponse::InternalServerError().json(&ErrorResponse {
                error: error.to_string(),
            });
        }
        _ => {}
    }

    // Check user id
    let user_id = req.user.id.as_str();
    // match state.users.get_user(user_id).await ...

    // Check that user has authorized the partner ...
    match state
        .authorized_partners_by_user
        .is_authorized(user_id, partner_id)
        .await
    {
        Ok(false) => {
            return HttpResponse::Forbidden().json(&ErrorResponse {
                error: String::from("User has not autorized partner."),
            })
        }
        Err(error) => {
            return HttpResponse::InternalServerError().json(&ErrorResponse {
                error: error.to_string(),
            })
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
    let server_binding_address = config
        .get_str("server.bind")
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::Other))?;

    let state = web::Data::new(AppState {
        sender,
        partners: PartnersBuilder::build(&config).unwrap(),
        authorized_partners_by_user: AuthorizedPartnersByUserBuilder::build(&config).unwrap(),
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
    .bind(server_binding_address)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use crate::ucdp::api::User;
    use crate::ucdp::dal::{AuthorizedPartnersByUserDao, Partner, PartnersDao, PartnersError};
    use crate::ucdp::web::{proxy, AppState};
    use actix_http::http::Method;
    use actix_web::dev::{Service, ServiceResponse};
    use actix_web::http::StatusCode;
    use actix_web::test::{init_service, TestRequest};
    use actix_web::{web, App};
    use async_trait::async_trait;
    use crossbeam_channel::unbounded;

    struct OptionPartnerDao {
        partner: Option<Partner>,
    }

    #[async_trait]
    impl PartnersDao for OptionPartnerDao {
        async fn get_partner(&self, p: &str) -> Result<Partner, PartnersError> {
            self.partner
                .clone()
                .ok_or_else(|| PartnersError::PartnerNotFound(p.to_string()))
        }
    }

    struct AuthorizedPartnerByUser {
        is_partner_authorized: bool,
    }

    #[async_trait]
    impl AuthorizedPartnersByUserDao for AuthorizedPartnerByUser {
        async fn is_authorized(
            &self,
            _: &str,
            _: &str,
        ) -> Result<bool, crate::ucdp::dal::AuthorizedPartnersByUserError> {
            Ok(self.is_partner_authorized)
        }
    }

    async fn get_response(
        partner: Option<Partner>,
        is_partner_authorized: bool,
    ) -> ServiceResponse {
        let (sender, _) = unbounded::<crate::ucdp::stream::Events>();
        let state = web::Data::new(AppState {
            sender,
            partners: Box::new(OptionPartnerDao { partner }),
            authorized_partners_by_user: Box::new(AuthorizedPartnerByUser {
                is_partner_authorized,
            }),
        });
        let service = init_service(App::new().app_data(state.clone()).service(proxy)).await;
        let request = TestRequest::default()
            .uri("/v1/events")
            .method(Method::POST)
            .set_json(&crate::ucdp::api::Events {
                partner: "0x123456789".into(),
                user: User {
                    id: "0x9876543210".into(),
                },
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
        let response = get_response(
            Some(Partner {
                name: "".into(),
                enabled: true,
            }),
            true,
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn http_server_simple_request_err_no_partner() {
        let response = get_response(None, true).await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_rt::test]
    async fn http_server_simple_request_err_partner_disabled() {
        let response = get_response(
            Some(Partner {
                name: "".into(),
                enabled: false,
            }),
            true,
        )
        .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[actix_rt::test]
    async fn http_server_simple_request_err_partner_not_authorized_by_user() {
        let response = get_response(
            Some(Partner {
                name: "".into(),
                enabled: true,
            }),
            false,
        )
        .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
