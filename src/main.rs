mod api;
pub mod db;
use actix_web::{App, HttpServer};
use api::api_service;
use dotenv::dotenv;
use sea_orm::Database;
use std::env;
use utoipa_actix_web::AppExt;
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let conn = Database::connect(env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to DB");

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
