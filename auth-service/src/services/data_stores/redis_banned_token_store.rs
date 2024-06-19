use crate::{
    domain::data_stores::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};
use color_eyre::eyre::{Context, Result};
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
    #[tracing::instrument(name = "RedisBannedTokenStore:: Add Token", skip_all)]
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        let key = get_key(&token);

        let token_ttl_seconds: u64 = TOKEN_TTL_SECONDS
            .try_into()
            .wrap_err("Failed to cast TOKEN_TTL_SECONDS to u64")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        let _: () = self
            .conn
            .write()
            .await
            .set_ex(key, true, token_ttl_seconds)
            .wrap_err("Failed to set banned token in Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        Ok(())
    }

    #[tracing::instrument(name = "RedisBannedTokenStore:: Contains Token", skip_all)]
    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let key = get_key(token);
        let is_banned: bool = self
            .conn
            .write()
            .await
            .exists::<String, bool>(key)
            .wrap_err("Failed to check if token exists in Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        Ok(is_banned)
    }

    #[tracing::instrument(name = "RedisBannedTokenStore:: Empty Store", skip_all)]
    async fn empty_store(&mut self) -> Result<(), BannedTokenStoreError> {
        let mut redis_connection = self.conn.write().await;

        // @todo: implement with query_async instead
        redis::cmd("FLUSHDB")
            .arg("ASYNC")
            .query::<()>(&mut redis_connection)
            .wrap_err("Failed to flush Redis database")
            .map_err(BannedTokenStoreError::UnexpectedError)
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
