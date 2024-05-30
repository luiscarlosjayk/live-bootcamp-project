use crate::domain::data_stores::{BannedTokenStore, BannedTokenStoreError};
use async_trait::async_trait;
use std::collections::HashSet;

#[derive(Default, Debug)]
pub struct HashsetBannedTokenStore {
    pub tokens: HashSet<String>,
}

#[async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        if self.tokens.contains(&token) {
            return Ok(()); // No operation
        }

        self.tokens.insert(token);

        Ok(())
    }

    async fn validate_token(&self, token: String) -> Result<(), BannedTokenStoreError> {
        if self.tokens.contains(&token) {
            return Err(BannedTokenStoreError::InvalidToken);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use crate::domain::Email;
    use crate::utils::auth;
    use crate::utils::constants::env::JWT_SECRET_ENV_VAR;

    const DEFAULT_EMAIL: &str = "testing@email.com";

    #[tokio::test]
    async fn test_add_token() {
        env::set_var(JWT_SECRET_ENV_VAR, "foo");
        let mut banned_token_store = HashsetBannedTokenStore::default();
        let email = Email::parse(DEFAULT_EMAIL.to_string()).unwrap();
        let token = auth::generate_auth_token(&email).unwrap();

        // Test adding a new token
        let result = banned_token_store.add_token(token.clone()).await;
        assert!(result.is_ok());

        // Test adding an existing token
        let result = banned_token_store.add_token(token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_succeeds_if_token_is_not_banned() {
        let banned_token_store = HashsetBannedTokenStore::default();
        let token = "random_token".to_string();

        let result = banned_token_store.validate_token(token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_fails_if_token_is_banned() {
        let mut banned_token_store = HashsetBannedTokenStore::default();
        let token = "random_token".to_string();

        let result = banned_token_store.add_token(token.clone()).await;
        assert!(result.is_ok());

        let result = banned_token_store.validate_token(token).await;
        assert_eq!(result, Err(BannedTokenStoreError::InvalidToken));
    }
}
