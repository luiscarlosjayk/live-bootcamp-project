use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    User,
};

pub struct HashmapUserStore {
    pub users: HashMap<String, User>,
}

// DONE: Create a new struct called `HashmapUserStore` containing a `users` field
// which stores a `HashMap`` of email `String`s mapped to `User` objects.
// Derive the `Default` trait for `HashmapUserStore`.
#[async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            self.users.insert(user.email.clone(), user);
            Ok(())
        }
    }

    // DONE: Implement a public method called `get_user`, which takes an
    // immutable reference to self and an email string slice as arguments.
    // This function should return a `Result` type containing either a
    // `User` object or a `UserStoreError`.
    // Return `UserStoreError::UserNotFound` if the user can not be found.
    async fn get_user(&self, email: String) -> Result<User, UserStoreError> {
        match self.users.get(&email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    // DONE: Implement a public method called `validate_user`, which takes an
    // immutable reference to self, an email string slice, and a password string slice
    // as arguments. `validate_user` should return a `Result` type containing either a
    // unit type `()` if the email/password passed in match an existing user, or a `UserStoreError`.
    // Return `UserStoreError::UserNotFound` if the user can not be found.
    // Return `UserStoreError::InvalidCredentials` if the password is incorrect.
    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email.to_string()).await?;

        if user.password != password {
            return Err(UserStoreError::InvalidCredentials);
        }

        Ok(())
    }
}

// DONE: Add unit tests for your `HashmapUserStore` implementation
#[cfg(test)]
mod tests {
    use super::*;

    impl Default for HashmapUserStore {
        fn default() -> Self {
            let user = User::new(
                "default_email".to_owned(),
                "default_password".to_owned(),
                true,
            );
            let mut users = HashMap::new();
            users.insert(user.email.clone(), user);
            Self { users }
        }
    }

    #[tokio::test]
    async fn test_add_user() {
        let mut hashmap_user_store = HashmapUserStore::default();
        let email = "another_email".to_owned();
        let password = "password".to_owned();
        let requires_2fa = true;
        let added_user = hashmap_user_store
            .add_user(User {
                email,
                password,
                requires_2fa,
            })
            .await;
        assert!(added_user.is_ok());
    }

    #[tokio::test]
    async fn test_get_user() {
        let default_email = "default_email".to_owned();
        let default_password = "default_password".to_owned();
        let default_requires_2fa = true;
        let hashmap_user_store = HashmapUserStore::default();

        let user = hashmap_user_store.get_user(default_email.clone()).await;
        let user = user.expect("Couldn't retrieve user");

        assert_eq!(user.email, default_email);
        assert_eq!(user.password, default_password);
        assert_eq!(user.requires_2fa, default_requires_2fa);
    }

    #[tokio::test]
    async fn test_validate_user() {
        // Success use case
        let default_email = "default_email".to_owned();
        let default_password = "default_password".to_owned();
        let hashmap_user_store = HashmapUserStore::default();

        let user = hashmap_user_store.get_user(default_email.clone()).await;
        let user = user.expect("Couldn't retrieve default user");

        assert_eq!(user.password, default_password);

        // Should return UserStoreError::UserNotFound
        let invalid_user = hashmap_user_store.get_user("invalid".to_owned()).await;
        assert_eq!(invalid_user.unwrap_err(), UserStoreError::UserNotFound);

        // Should return UserStoreError::InvalidCredentials
        let invalid = hashmap_user_store
            .validate_user(&default_email.clone(), "invalid")
            .await;
        assert_eq!(invalid.unwrap_err(), UserStoreError::InvalidCredentials);
    }
}
