pub mod tweet_db;
use std::cmp::max;

use crate::db::nick;
use redis::JsonAsyncCommands;
use redis::aio::MultiplexedConnection;
use redis::{AsyncTypedCommands, aio::ConnectionManager};
use sea_orm::{DatabaseConnection, DbErr};
use sea_orm::{EntityTrait, QueryOrder};
use serde::{Deserialize, Serialize};
use tweet_db::{GetTweet, PostTweet, PostTweetApiResponse};
use utoipa::ToSchema;

use crate::db::tweet;
#[derive(Clone)]
pub struct Storage {
    db: DatabaseConnection,
    redis: ConnectionManager,
}
#[derive(Serialize, ToSchema)]
pub struct GetTweetResponse {
    body: Vec<GetTweet>,
    next_cursor: Option<i32>,
}

const page_size: u32 = 32;
impl Storage {
    pub fn new(db: DatabaseConnection, redis: ConnectionManager) -> Self {
        Self { db, redis }
    }
    pub async fn insert_tweet(
        &mut self,
        post: &PostTweet,
        nick_id: Option<i32>,
    ) -> Result<PostTweetApiResponse, DbErr> {
        let tweet_resp = tweet_db::post_tweet_db(post, &self.db, nick_id).await;

        tweet_resp.map(|x| {
            self.push_tweet_cache(&x.body);
            x.api_response
        })
    }
    pub async fn get_tweet(&mut self, cursor: Option<u32>) -> Result<GetTweetResponse, DbErr> {
        if cursor.is_none() {
            if let Ok(Some(latest_id)) = self.redis.get_int("tweet_latest_id").await {
                if let Ok(tweets_cache) = self
                    .redis
                    .mget(
                        (latest_id..latest_id - page_size as isize)
                            .map(|x| format!("tweet:{}", x))
                            .collect::<Vec<String>>()
                            .join(" "),
                    )
                    .await
                {
                    if !tweets_cache.iter().any(|x| x.is_none()) {
                        let cached_deserial = tweets_cache
                            .iter()
                            .map(|x| x.to_owned().unwrap())
                            .map(|x| serde_json::from_str(x.as_str()).unwrap())
                            .collect::<Vec<GetTweet>>();
                        println!("Redis Cache Hit");
                        return Ok(GetTweetResponse {
                            body: cached_deserial,
                            next_cursor: Some(latest_id as i32 - page_size as i32),
                        });
                    }
                }
            }
        }
        println!("Fetch By DB");
        let tweets = tweet_db::get_tweet_db(cursor, page_size, &self.db).await?;
        for i in tweets.iter() {
            // self.push_tweet_cache(i)
        }

        Ok(GetTweetResponse {
            next_cursor: tweets.last().map(|x| x.id).filter(|x| *x > 1),
            body: tweets,
        })
    }
    async fn push_tweet_cache(&mut self, tweet: GetTweet) -> redis::RedisResult<()> {
        let format = serde_json::to_string(&tweet);

        if let Ok(tweet_str) = format {
            let key = format!("tweet:{}", tweet.id);
            let _: () = self.redis.json_set(&key, "$", &tweet_str).await?;
            self.redis.expire(&key, 360).await?;
            // max id swap
            // self.redis.set("tweet_latest_id", tweet.id).await?;
            let _: () = redis::transaction(
                &self.redis,
                &["tweet_latest_id"],
                |con: &ConnectionManager, pipe| {
                    let old = con.get_int("tweet_latest_id")?;
                    pipe.set("tweet_latest_id", max(old, tweet.id))
                        .ignore()
                        .query(con)
                },
            )?;
        }
        Ok(())
    }
    async fn get_nick_name_db(&self, nick_id: i32) -> Result<Option<String>, DbErr> {
        nick::Entity::find_by_id(nick_id)
            .one(&self.db)
            .await
            .map(|x| x.map(|y| y.name))
    }
    pub async fn warmup(&mut self) -> Result<(), DbErr> {
        if let Some(latest) = tweet::Entity::find()
            .order_by_desc(tweet::Column::Id)
            .one(&self.db)
            .await?
        {
            let _ = self.redis.set("tweet_latest_id", latest.id).await;
        }
        Ok(())
    }
}
