use crate::{api::{CommonErrEnum, CommonErrStruct}, storage::{tweet_db::PostTweetApiResponse, GetTweetResponse}};
use actix_web::{
     get,  post, web::{self, Json}, HttpRequest, Responder, Result
};
use serde::Deserialize;
use crate::storage::tweet_db::PostTweet;

#[derive(Deserialize)]
struct GetTweetQuery{
    cursor: Option<i32>
}

#[utoipa::path(responses(
        (status = 200, body = GetTweetResponse), 
        (status = 500, description = "Internal Server Error",body=CommonErrStruct)
    ),params(
        ("cursor" = Option<u32>,Query, description="cursor"),
    )
)]
#[get("/tweet")]
pub async fn get_tweet(data: web::Data<super::AppContext>, query: web::Query<GetTweetQuery>) -> Result<impl Responder, CommonErrEnum>{
    let db_resp = data.storage.to_owned().get_tweet(query.cursor).await;
    db_resp.map(Json).map_err(CommonErrEnum::from)
}

#[utoipa::path(responses(
        (status = 200, description = "Success!", body=PostTweetApiResponse), 
        (status = 500, description = "Internal Server Error",body=CommonErrStruct),
        (status = 400, description = "Bad Request Body, reason in response body",body=CommonErrStruct)
    ),
)]
#[post("/tweet")]
pub async fn post_tweet(tweet: web::Json<PostTweet>, data: web::Data<super::AppContext>, _req: HttpRequest) -> Result<impl Responder,CommonErrEnum>{
    // check nick is available
    // set limit

    // proccess
    data.storage
        .to_owned()
        .insert_tweet(&tweet.0, None).await.map(Json).map_err(CommonErrEnum::from)
}
