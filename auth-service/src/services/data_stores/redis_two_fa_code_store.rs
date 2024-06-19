use chrono::Duration;
use color_eyre::eyre::Context;
use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
    #[tracing::instrument(name = "RedisTwoFACodeStore:: Add Code", skip_all)]
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(&email);

        let data = TwoFATuple(
            login_attempt_id.as_ref().to_string(),
            code.as_ref().to_string(),
        );
        let serialized_data = serde_json::to_string(&data)
            .wrap_err("Failed to serialize 2Fa tuple")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;

        let ttl_in_seconds: u64 = TEN_MINUTES_IN_SECONDS
            .try_into()
            .wrap_err("Failed to cast 10 minutes representation from i64 to u64")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;

        let _: () = self
            .conn
            .write()
            .await
            .set_ex::<String, String, ()>(key, serialized_data, ttl_in_seconds)
            .wrap_err("Failed to set 2FA code in Redis")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    #[tracing::instrument(name = "RedisTwoFACodeStore:: Remove Code", skip_all)]
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(email);
        let _: () = self
            .conn
            .write()
            .await
            .del::<String, ()>(key)
            .wrap_err("Failed to delete 2FA code from Redis")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    #[tracing::instrument(name = "RedisTwoFACodeStore:: Get Code", skip_all)]
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let key = get_key(email);

        let two_fa_tuple = match self
            .conn
            .write()
            .await
            .get::<String, String>(key)
            .map_err(|_| TwoFACodeStoreError::LoginAttemptIdNotFound)
        {
            Ok(value) => serde_json::from_str::<TwoFATuple>(&value)
                .wrap_err("Failed to deserialize 2FA tuple")
                .map_err(|_| TwoFACodeStoreError::LoginAttemptIdNotFound)?,
            Err(_) => return Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        };

        let TwoFATuple(login_attempt_id, two_fa_code) = two_fa_tuple;

        let login_attempt_id = LoginAttemptId::parse(login_attempt_id)
            .map_err(TwoFACodeStoreError::UnexpectedError)?;
        let two_fa_code =
            TwoFACode::parse(two_fa_code).map_err(TwoFACodeStoreError::UnexpectedError)?;

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
