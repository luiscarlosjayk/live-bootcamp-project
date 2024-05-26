use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    Email, Password, User,
};

#[derive(Default)]
pub struct HashmapUserStore {
    pub users: HashMap<Email, User>,
}

// DONE: Create a new struct called `HashmapUserStore` containing a `users` field
// which stores a `HashMap`` of email `String`s mapped to `User` objects.
// Derive the `Default` trait for `HashmapUserStore`.
#[async_trait]
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

    // DONE: Implement a public method called `get_user`, which takes an
    // immutable reference to self and an email string slice as arguments.
    // This function should return a `Result` type containing either a
    // `User` object or a `UserStoreError`.
    // Return `UserStoreError::UserNotFound` if the user can not be found.
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    // DONE: Implement a public method called `validate_user`, which takes an
    // immutable reference to self, an email string slice, and a password string slice
    // as arguments. `validate_user` should return a `Result` type containing either a
    // unit type `()` if the email/password passed in matches an existing user, or a `UserStoreError`.
    // Return `UserStoreError::UserNotFound` if the user can not be found.
    // Return `UserStoreError::InvalidCredentials` if the password is incorrect.
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

// DONE: Add unit tests for your `HashmapUserStore` implementation
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Email, Password};

    const DEFAULT_EMAIL: &str = "testing@email.com";
    const DEFAULT_PASSWORD: &str = "qazWSX123";

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
}
