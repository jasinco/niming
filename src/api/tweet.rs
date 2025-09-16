use crate::{api::{CommonErrEnum, CommonErrStruct}, db::{nick, tweet}};
use actix_session::Session;
use actix_web::{
    error, get,  post, web::{self, Json}, Error, HttpRequest, Responder, Result
};
use base64ct::{Base64, Encoding};
use sea_orm::{DbErr, entity::*, query::*};
use serde::{Deserialize, Serialize};
use blake2::{Blake2b512, Digest};
// webapi interface
const page_size: u32 = 32;

#[derive(utoipa::ToSchema, Serialize)]
pub struct GetTweet {
    pub id: i32,
    pub content: String,
    pub post_at: chrono::DateTime<chrono::FixedOffset>,
    pub heart: i32,
    pub igid: Option<String>,
    pub nick_name: Option<String>,
}
impl From<&tweet::Model> for GetTweet {
    fn from(w: &tweet::Model) -> GetTweet{
        GetTweet {
            id: w.id,
            content: w.content.clone(),
            post_at: w.postat,
            heart: w.heart,
            igid: w.igid.clone(),
            nick_name: None,
        }
    }
}
impl From<&(tweet::Model, Option<nick::Model>)> for GetTweet{
    fn from(w: &(tweet::Model, Option<nick::Model>)) -> GetTweet{
        let mut _tweet: GetTweet = (&w.0).into();
        if let Some(nick) = w.1.clone() {
            _tweet.nick_name = Some(nick.name);
        }
        return _tweet;
    }
}

#[derive(utoipa::ToSchema, Deserialize)]
pub struct PostTweet {
    pub content: String,
    pub with_nick: bool,
    pub media_quantity: i8,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct PostTweetResponse {
    pub id: i32, 
    pub hash: String, 
}

async fn get_tweet_db(page: u32, db: &sea_orm::DbConn) -> Result<Vec<GetTweet>, DbErr> {
    Ok(tweet::Entity::find()
        .order_by_desc(tweet::Column::Id)
        .find_also_related(nick::Entity)
        .cursor_by(tweet::Column::Id)
        .after(page * page_size)
        .before((page + 1) * page_size)
        .all(db)
        .await?
        .iter().map(GetTweet::from).collect()
    )
}


async fn post_tweet_db(post: &PostTweet, db: &sea_orm::DbConn, nick_id: Option<i32>) -> Result<PostTweetResponse,DbErr>{
    let mut hasher = Blake2b512::new();
    hasher.update(&post.content);
    hasher.update(chrono::Utc::now().to_string());
    let hash = Base64::encode_string(&hasher.finalize());

    
    let post = tweet::ActiveModel{
        content: Set(post.content.to_owned()),
        nick_id: Set(nick_id),
        hash: Set(hash),
        ..Default::default()
    };
    let post_ret = post.insert(db).await?;


    Ok(PostTweetResponse{
        id:post_ret.id,
        hash:post_ret.hash
    })
}


#[utoipa::path(responses(
        (status = 200, body = GetTweet), 
        (status = 500, description = "Internal Server Error",body=CommonErrStruct)
    )
)]
#[get("/tweet")]
pub async fn get_tweet(data: web::Data<super::AppContext>, _req: HttpRequest) -> Result<impl Responder, CommonErrEnum>{
    let db_resp= get_tweet_db(0, &data.db).await;
    db_resp.map(Json).map_err(CommonErrEnum::from)
}

#[utoipa::path(responses(
        (status = 200, description = "Success!", body=PostTweetResponse), 
        (status = 500, description = "Internal Server Error",body=CommonErrStruct),
        (status = 400, description = "Bad Request Body, reason in response body",body=CommonErrStruct)
    ),
)]
#[post("/tweet")]
pub async fn post_tweet(tweet: web::Json<PostTweet>, session:Session,data: web::Data<super::AppContext>, _req: HttpRequest) -> Result<impl Responder,CommonErrEnum>{
    // check nick is available
    // set limit
    let session_id = session.get::<i32>("session_id").map_err(CommonErrEnum::from)?;
    if tweet.with_nick && session_id.is_none(){
        return Err(CommonErrEnum::NoLoginUseNick);
    }

    // proccess
    let resp = post_tweet_db(&tweet, &data.db, session_id).await;
    resp.map(Json).map_err(CommonErrEnum::from)
}
