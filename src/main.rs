mod api;
pub mod db;
pub mod imgconv;
pub mod storage;
use actix_web::{App, HttpServer, web};
use api::api_service;
use dotenv::dotenv;
use fast_log::Config;
use log::{error, info, warn};
use moka::future::Cache;
use sea_orm::{ConnectOptions, Database};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use utoipa::openapi::Contact;
use utoipa_actix_web::AppExt;
use utoipa_swagger_ui::SwaggerUi;

use crate::storage::Storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    // set logger
    let log_cfg = Config::new()
        .console()
        .chan_len(Some(100000))
        .level(log::LevelFilter::Debug);
    fast_log::init(log_cfg).unwrap();

    let mut conn_opt = ConnectOptions::new(env::var("DATABASE_URL").unwrap());
    conn_opt
        .max_connections(100)
        .min_connections(10)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(8))
        .sqlx_logging(true);

    let conn = Database::connect(conn_opt).await.expect("DB not available");

    // moka cache
    let cache = Cache::builder()
        .max_capacity(
            env::var("CACHE_SIZE")
                .map(|x| x.parse().unwrap())
                .unwrap_or(10_000),
        )
        .time_to_live(Duration::from_secs(20 * 60))
        .time_to_idle(Duration::from_secs(5 * 60))
        .build_with_hasher(ahash::RandomState::default());

    let ctx = api::AppContext {
        storage: Storage::new(conn, cache),
    };

    println!("HTTP Server Starting");

    HttpServer::new(move || {
        App::new()
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
    // let mut handles = vec![];
    // for _ in 0..50 {
    //     let mut stg = ctx.storage.clone();
    //     handles.push(task::spawn(async move {
    //         stg.get_tweet(None).await;
    //     }))
    // }
    // for h in handles {
    //     h.await;
    // }
    // Ok(())
}
