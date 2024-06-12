use std::sync::Arc;

use chrono::Duration;
use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    Email,
};

pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        // TODO:
        let key = get_key(&email);
        let two_fa_tuple = TwoFATuple(
            login_attempt_id.as_ref().to_string(),
            code.as_ref().to_string(),
        );
        let value = serde_json::to_string(&two_fa_tuple)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
        let ttl_in_seconds: u64 = TEN_MINUTES_IN_SECONDS
            .try_into()
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(self
            .conn
            .write()
            .await
            .set_ex::<String, String, ()>(key, value, ttl_in_seconds)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?)
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(email);
        self.conn
            .write()
            .await
            .del::<String, ()>(key)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let key = get_key(email);
        dbg!(&key);
        let two_fa_tuple = match self
            .conn
            .write()
            .await
            .get::<String, String>(key)
            .map_err(|_| TwoFACodeStoreError::LoginAttemptIdNotFound)
        {
            Ok(t) => serde_json::from_str::<TwoFATuple>(&t)
                .map_err(|_| TwoFACodeStoreError::LoginAttemptIdNotFound)?,
            Err(_) => return Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        };
        let TwoFATuple(login_attempt_id, two_fa_code) = two_fa_tuple;

        let login_attempt_id = LoginAttemptId::parse(login_attempt_id)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
        let two_fa_code =
            TwoFACode::parse(two_fa_code).map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok((login_attempt_id, two_fa_code))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: i64 = Duration::minutes(10).num_seconds();
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
