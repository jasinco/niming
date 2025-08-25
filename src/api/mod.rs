mod hello;
use utoipa_actix_web::scope;
use utoipa_actix_web::service_config::ServiceConfig;

pub fn api_service(config: &mut ServiceConfig) {
    config.service(hello::get_post);
}
