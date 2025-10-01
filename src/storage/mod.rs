pub mod tweet_db;
use crate::db::nick;
use ahash::{HashMap, HashMapExt};
use bincode::config::{self, Configuration};
use log::{debug, info};
use moka::future::Cache;
use parking_lot::RwLock;
use sea_orm::{DatabaseConnection, DbErr};
use sea_orm::{EntityTrait, QueryOrder};
use serde::Serialize;
use std::cmp::max;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use tokio::sync::Notify;
use tweet_db::{GetTweet, PostTweet, PostTweetApiResponse};
use utoipa::ToSchema;

use crate::db::tweet;
#[derive(Clone)]
pub struct Storage {
    db: DatabaseConnection,
    latest_id: Arc<AtomicI32>,
    cache: Cache<String, Vec<u8>, ahash::RandomState>,
    enc_cfg: Configuration,
    cache_thres: Arc<RwLock<HashMap<i32, Notify>>>,
}
#[derive(Serialize, ToSchema)]
pub struct GetTweetResponse {
    body: Vec<GetTweet>,
    next_cursor: Option<i32>,
}

const page_size: u32 = 32;
impl Storage {
    pub fn new(db: DatabaseConnection, cache: Cache<String, Vec<u8>, ahash::RandomState>) -> Self {
        Self {
            db,
            cache,
            latest_id: Arc::new(AtomicI32::new(0)),
            enc_cfg: config::standard(),
            cache_thres: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub async fn insert_tweet(
        &mut self,
        post: &PostTweet,
        nick_id: Option<i32>,
    ) -> Result<PostTweetApiResponse, DbErr> {
        let tweet_resp = tweet_db::post_tweet_db(post, &self.db, nick_id).await;

        for tweet in tweet_resp.iter() {
            let mut get_tweet = GetTweet::from(&tweet.body);
            if let Some(nick_id) = tweet.body.nick_id {
                if let Ok(nick_name) = self.get_nick_name_db(nick_id).await {
                    get_tweet.nick_name = nick_name;
                    let _ = self.push_tweet_cache(&get_tweet).await;
                }
            }
        }
        tweet_resp.map(|x| x.api_response)
    }
    pub async fn get_tweet(&mut self, cursor: Option<i32>) -> Result<GetTweetResponse, DbErr> {
        for _ in 1..=2 {
            // get cache
            let latest_id = self.latest_id.load(Ordering::Acquire);
            let id_range = cursor
                .map(|x| {
                    ((x as u32).saturating_sub(page_size - 1)..=x as u32)
                        .rev()
                        .map(|a| a as i32)
                        .collect()
                })
                .unwrap_or(
                    ((latest_id as u32).saturating_sub(page_size - 1)..=latest_id as u32)
                        .map(|a| a as i32)
                        .rev()
                        .collect::<Vec<i32>>(),
                );
            let cached = self.get_tweet_cache(&id_range).await;
            info!("get cached, len:{}", cached.len());
            if cached.len() as u32 == page_size {
                return Ok(GetTweetResponse {
                    next_cursor: cached.last().map(|x| x.id),
                    body: cached,
                });
            }
            info!("Fetch By Cache");
        }

        info!("Fetch By DB");
        let tweets = tweet_db::get_tweet_db(cursor.map(|x| x as u32), page_size, &self.db).await?;
        let _ = self.push_tweets_cache(&tweets).await;

        Ok(GetTweetResponse {
            next_cursor: tweets.last().map(|x| x.id).filter(|x| *x > 1),
            body: tweets,
        })
    }
    async fn get_nick_name_db(&self, nick_id: i32) -> Result<Option<String>, DbErr> {
        nick::Entity::find_by_id(nick_id)
            .one(&self.db)
            .await
            .map(|x| x.map(|y| y.name))
    }
    async fn push_tweet_cache(&mut self, tweet: &GetTweet) {
        if let Ok(enc) = bincode::encode_to_vec(tweet, self.enc_cfg) {
            self.cache.insert(format!("twt:{}", tweet.id), enc).await;
        }
    }
    async fn push_tweets_cache(&mut self, tweets: &[GetTweet]) {
        for tweet in tweets {
            if let Ok(enc) = bincode::encode_to_vec(tweet, self.enc_cfg) {
                self.cache.insert(format!("twt:{}", tweet.id), enc).await;
            }
            for _ in 1..3 {
                if self
                    .latest_id
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
                        Some(max(x, tweet.id))
                    })
                    .is_ok()
                {
                    break;
                }
            }
        }
    }

    async fn get_tweet_cache(&mut self, id_range: &[i32]) -> Vec<GetTweet> {
        let keys = id_range.iter().map(|x| format!("twt:{}", x));
        let mut cached: Vec<GetTweet> = vec![];
        info!("keys_len: {}", keys.len());
        for key in keys {
            info!("key: {}", key);
            if let Some(single) = self.cache.get(&key).await {
                if let Ok(decoded) = bincode::decode_from_slice(&single, self.enc_cfg) {
                    cached.push(decoded.0);
                }
            }
        }
        cached
    }
}
