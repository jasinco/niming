mod api;
pub mod db;
pub mod imgconv;
pub mod storage;
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_web::cookie::Key;
use actix_web::{App, HttpServer, web};
use api::api_service;
use dotenv::dotenv;
use redis::aio::ConnectionManager;
use sea_orm::{ConnectOptions, Database};
use std::env;
use std::time::Duration;
use utoipa::openapi::Contact;
use utoipa_actix_web::AppExt;
use utoipa_swagger_ui::SwaggerUi;

use crate::storage::Storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let mut conn_opt = ConnectOptions::new(env::var("DATABASE_URL").unwrap());
    conn_opt
        .max_connections(100)
        .min_connections(10)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(8))
        .sqlx_logging(true);

    let conn = Database::connect(conn_opt).await.expect("DB not available");

    let redis_url = env::var("REDIS_URL").unwrap();
    let redis_client = redis::Client::open(redis_url.clone()).expect("Redis Connection Failed");
    let redis_con_manager = ConnectionManager::new(redis_client)
        .await
        .expect("Redis Connection Manager Failed");
    let secret_key = Key::generate();
    let store = RedisSessionStore::new(redis_url)
        .await
        .expect("Redis Connect");

    let mut ctx = api::AppContext {
        storage: Storage::new(conn, redis_con_manager),
    };
    // Redis WarmUp
    let _ = ctx.storage.warmup().await;

    println!("HTTP Server Starting");

    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(store.clone(), secret_key.clone()))
            .app_data(web::Data::new(ctx.clone()))
            .into_utoipa_app()
            .service(utoipa_actix_web::scope("/api/v1").configure(api_service))
            .openapi_service(|mut api| {
                api.info.title = "中工匿名".to_string();
                api.info.description = Some("中工匿名API".to_string());
                api.info.contact = Some(
                    Contact::builder()
                        .name(Some("jasinco"))
                        .email(Some("jasinc90@gmail.com"))
                        .build(),
                );
                SwaggerUi::new("/api/swagger-ui/{_:.*}").url("/api/openapi.json", api)
            })
            .into_app()
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
