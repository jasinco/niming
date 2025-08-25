mod api;
use actix_web::{App, HttpServer};
use api::api_service;
use utoipa_actix_web::{AppExt, scope};
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .into_utoipa_app()
            .service(utoipa_actix_web::scope("/api/v1").configure(api_service))
            .openapi_service(|api| {
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api/openapi.json", api)
            })
            .into_app()
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
