use crate::db::{nick, tweet};
use base64ct::{Base64, Encoding};
use bincode::{Decode, Encode};
use blake2::{Blake2b512, Digest};
use sea_orm::{DbErr, entity::*, query::*};
use serde::{Deserialize, Serialize};

#[derive(utoipa::ToSchema, Serialize, Deserialize, Encode, Decode)]
pub struct GetTweet {
    pub id: i32,
    pub content: String,
    #[bincode(with_serde)]
    pub post_at: chrono::DateTime<chrono::FixedOffset>,
    pub heart: i32,
    pub igid: Option<String>,
    pub nick_name: Option<String>,
}
impl From<&tweet::Model> for GetTweet {
    fn from(w: &tweet::Model) -> GetTweet {
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
impl From<tweet::Model> for GetTweet {
    fn from(w: tweet::Model) -> GetTweet {
        GetTweet {
            id: w.id,
            content: w.content,
            post_at: w.postat,
            heart: w.heart,
            igid: w.igid,
            nick_name: None,
        }
    }
}
impl From<&(tweet::Model, Option<nick::Model>)> for GetTweet {
    fn from(w: &(tweet::Model, Option<nick::Model>)) -> GetTweet {
        let mut _tweet: GetTweet = (&w.0).into();
        if let Some(nick) = w.1.clone() {
            _tweet.nick_name = Some(nick.name);
        }
        _tweet
    }
}
impl From<(tweet::Model, Option<nick::Model>)> for GetTweet {
    fn from(w: (tweet::Model, Option<nick::Model>)) -> GetTweet {
        let mut _tweet: GetTweet = w.0.into();
        if let Some(nick) = w.1 {
            _tweet.nick_name = Some(nick.name);
        }
        _tweet
    }
}

#[derive(utoipa::ToSchema, Deserialize)]
pub struct PostTweet {
    pub content: String,
    pub with_nick: bool,
    pub media_quantity: i8,
}

#[derive(utoipa::ToSchema, Serialize)]
pub struct PostTweetApiResponse {
    pub id: i32,
    pub hash: String,
}

pub struct PostTweetResponse {
    pub api_response: PostTweetApiResponse,
    pub body: tweet::Model,
}

pub async fn get_tweet_db(
    cursor: Option<u32>,
    length: u32,
    db: &sea_orm::DbConn,
) -> Result<Vec<GetTweet>, DbErr> {
    let mut _cursor = tweet::Entity::find()
        .find_also_related(nick::Entity)
        .cursor_by(tweet::Column::Id)
        .order_by_desc(tweet::Column::Id);
    if let Some(ptr) = cursor {
        _cursor.before(ptr).after(ptr.saturating_sub(length));
    } else {
        _cursor.last(length as u64);
    }
    let mut response = _cursor
        .all(db)
        .await?
        .iter()
        .map(GetTweet::from)
        .collect::<Vec<GetTweet>>();
    response.reverse();
    Ok(response)
}
pub async fn get_tweet_db_single(id: u32, db: &sea_orm::DbConn) -> Result<Option<GetTweet>, DbErr> {
    let single = tweet::Entity::find()
        .order_by_desc(tweet::Column::Id)
        .find_also_related(nick::Entity)
        .filter(tweet::Column::Id.eq(id))
        .one(db)
        .await?;
    Ok(single.map(|x| GetTweet::from(&x)))
}

pub async fn post_tweet_db(
    post: &PostTweet,
    db: &sea_orm::DbConn,
    nick_id: Option<i32>,
) -> Result<PostTweetResponse, DbErr> {
    let mut hasher = Blake2b512::new();
    hasher.update(&post.content);
    hasher.update(chrono::Utc::now().to_string());
    let hash = Base64::encode_string(&hasher.finalize());

    let post = tweet::ActiveModel {
        content: Set(post.content.to_owned()),
        nick_id: Set(nick_id),
        hash: Set(hash),
        ..Default::default()
    };
    let post_ret = post.insert(db).await?;

    Ok(PostTweetResponse {
        api_response: PostTweetApiResponse {
            id: post_ret.id,
            hash: post_ret.hash.clone(),
        },
        body: post_ret,
    })
}
