mod tweet;

use actix_session::SessionGetError;
use actix_web::{HttpResponse, Responder, error, http::StatusCode, web::Json};
use sea_orm::{DatabaseConnection, DbErr};
use serde::Serialize;
use strum::Display;
use utoipa::ToSchema;
use utoipa_actix_web::service_config::ServiceConfig;

use crate::storage::Storage;

pub fn api_service(config: &mut ServiceConfig) {
    // config.service(hello::get_post);
    config.service(tweet::get_tweet);
    config.service(tweet::post_tweet);
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct AppContext {
    pub storage: Storage,
}
#[derive(Debug, Display, Serialize, Clone)]
pub enum CommonErrEnum {
    DbErr = 1,
    InproperContentLength = 2,
    ExceedImageNumLimit = 3,
    NoLoginUseNick = 4,
}

#[derive(Serialize, ToSchema)]
pub struct CommonErrStruct {
    code: i16,
    reason: String,
}

impl From<DbErr> for CommonErrEnum {
    fn from(value: DbErr) -> Self {
        // impl log
        println!("DBErr {}", value.to_string());
        Self::DbErr
    }
}
impl From<SessionGetError> for CommonErrEnum {
    fn from(value: SessionGetError) -> Self {
        // impl log
        Self::NoLoginUseNick
    }
}
impl error::ResponseError for CommonErrEnum {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(CommonErrStruct {
            code: self.clone() as i16,
            reason: self.to_string(),
        })
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            CommonErrEnum::DbErr => StatusCode::INTERNAL_SERVER_ERROR,
            CommonErrEnum::InproperContentLength => StatusCode::BAD_REQUEST,
            CommonErrEnum::ExceedImageNumLimit => StatusCode::BAD_REQUEST,
            CommonErrEnum::NoLoginUseNick => StatusCode::BAD_REQUEST,
        }
    }
}
