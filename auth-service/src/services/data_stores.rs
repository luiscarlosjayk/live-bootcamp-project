use crate::domain::{
    data_stores::{
        BannedTokenStore, BannedTokenStoreError, LoginAttemptId, TwoFACode, TwoFACodeStore,
        TwoFACodeStoreError, UserStore, UserStoreError,
    },
    Email, Password, User,
};
use std::collections::HashMap;
use std::collections::HashSet;

/**
 * TwoFACodeStore
 */

#[derive(Default, Debug)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let value = (login_attempt_id, code);
        self.codes.insert(email, value);

        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        match self.codes.remove(email) {
            Some(_) => Ok(()),
            None => Err(TwoFACodeStoreError::UnexpectedError),
        }
    }

    async fn get_code(
        &self,
        email: Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        match self.codes.get(&email) {
            Some(code) => Ok(code.clone()),
            None => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }
}

/**
 * UserStore
 */

#[derive(Default, Debug)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }
        self.users.insert(user.email.clone(), user);
        Ok(())
    }

    async fn delete_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            self.users.remove(&user.email);

            return Ok(());
        }

        Err(UserStoreError::UserNotFound)
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(user) => {
                if user.password.eq(password) {
                    Ok(())
                } else {
                    Err(UserStoreError::InvalidCredentials)
                }
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }
}

/**
 * BannedTokenStore
 */
#[derive(Default, Debug)]
pub struct HashsetBannedTokenStore {
    tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        self.tokens.insert(token);
        Ok(())
    }

    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.contains(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStoreError},
        Email,
    };

    const DEFAULT_EMAIL: &str = "testing@email.com";
    const DEFAULT_PASSWORD: &str = "qazWSX123";
    /**
     * TwoFACodeStore tests
     */

    #[tokio::test]
    async fn test_add_2fa_code() {
        let mut two_fa_code_store = HashmapTwoFACodeStore::default();
        let email = Email::parse(DEFAULT_EMAIL.to_string()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let two_fa_code = TwoFACode::default();

        let result = two_fa_code_store
            .add_code(email, login_attempt_id, two_fa_code)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_2fa_code() {
        let mut two_fa_code_store = HashmapTwoFACodeStore::default();
        let email = Email::parse(DEFAULT_EMAIL.to_string()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        let result = two_fa_code_store
            .add_code(email.clone(), login_attempt_id, code)
            .await;
        assert!(result.is_ok());

        let result = two_fa_code_store.remove_code(&email).await;
        assert!(result.is_ok());

        let result = two_fa_code_store.remove_code(&email).await;
        assert_eq!(result, Err(TwoFACodeStoreError::UnexpectedError));
    }

    #[tokio::test]
    async fn test_get_2fa_code() {
        let mut two_fa_code_store = HashmapTwoFACodeStore::default();
        let email = Email::parse(DEFAULT_EMAIL.to_string()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        let result = two_fa_code_store
            .add_code(email.clone(), login_attempt_id.clone(), code.clone())
            .await;
        assert!(result.is_ok());

        let result = two_fa_code_store.get_code(email.clone()).await.unwrap();
        assert_eq!(result.0, login_attempt_id);
        assert_eq!(result.1, code);
    }

    /**
     * UserStore tests
     */

    #[tokio::test]
    async fn test_add_user() {
        let mut user_store = HashmapUserStore::default();
        let user = User {
            email: Email::parse(DEFAULT_EMAIL.to_owned()).unwrap(),
            password: Password::parse(DEFAULT_PASSWORD.to_owned()).unwrap(),
            requires_2fa: false,
        };

        // Test adding a new user
        let result = user_store.add_user(user.clone()).await;
        assert!(result.is_ok());

        // Test adding an existing user
        let result = user_store.add_user(user).await;
        assert_eq!(result, Err(UserStoreError::UserAlreadyExists));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse(DEFAULT_EMAIL.to_owned()).unwrap();

        let user = User {
            email: email.clone(),
            password: Password::parse(DEFAULT_PASSWORD.to_owned()).unwrap(),
            requires_2fa: false,
        };

        // Test getting a user that exists
        user_store.users.insert(email.clone(), user.clone());
        let result = user_store.get_user(&email).await;
        assert_eq!(result, Ok(user));

        // Test getting a user that doesn't exist
        let result = user_store
            .get_user(&Email::parse("nonexistent@example.com".to_owned()).unwrap())
            .await;

        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("test@example.com".to_owned()).unwrap();
        let password = Password::parse("abcDEF123".to_owned()).unwrap();

        let user = User {
            email: email.clone(),
            password: password.clone(),
            requires_2fa: false,
        };

        // Test validating a user that exists with correct password
        user_store.users.insert(email.clone(), user.clone());
        let result = user_store.validate_user(&email, &password).await;
        assert_eq!(result, Ok(()));

        // Test validating a user that exists with incorrect password
        let wrong_password = Password::parse("NONexistent123".to_owned()).unwrap();
        let result = user_store.validate_user(&email, &wrong_password).await;
        assert_eq!(result, Err(UserStoreError::InvalidCredentials));

        // Test validating a user that doesn't exist
        let result = user_store
            .validate_user(
                &Email::parse("nonexistent@example.com".to_string()).unwrap(),
                &password,
            )
            .await;

        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }

    /**
     * BannedTokenStore tests
     */

    #[tokio::test]
    async fn test_add_token() {
        let mut store = HashsetBannedTokenStore::default();
        let token = "test_token".to_owned();

        let result = store.add_token(token.clone()).await;

        assert!(result.is_ok());
        assert!(store.tokens.contains(&token));
    }

    #[tokio::test]
    async fn test_contains_token() {
        let mut store = HashsetBannedTokenStore::default();
        let token = "test_token".to_owned();
        store.tokens.insert(token.clone());

        let result = store.contains_token(&token).await;

        assert!(result.unwrap());
    }
}
