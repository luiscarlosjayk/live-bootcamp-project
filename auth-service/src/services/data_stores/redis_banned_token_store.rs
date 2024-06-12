use crate::{
    domain::data_stores::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};
use redis::{Commands, Connection};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        let key = get_key(&token);
        let token_ttl_seconds: u64 = TOKEN_TTL_SECONDS
            .try_into()
            .map_err(|_| BannedTokenStoreError::UnexpectedError)?;

        self.conn
            .write()
            .await
            .set_ex(key, true, token_ttl_seconds)
            .map_err(|_| BannedTokenStoreError::UnexpectedError)
    }

    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let key = get_key(token);
        self.conn
            .write()
            .await
            .exists::<String, bool>(key)
            .map_err(|_| BannedTokenStoreError::UnexpectedError)
    }

    async fn empty_store(&mut self) -> Result<(), BannedTokenStoreError> {
        let mut redis_connection = self.conn.write().await;

        // @todo: implement with query_async instead
        redis::cmd("FLUSHDB")
            .arg("ASYNC")
            .query::<()>(&mut redis_connection)
            .map_err(|_| BannedTokenStoreError::UnexpectedError)
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
