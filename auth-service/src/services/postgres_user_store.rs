use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    Email, Password, User,
};
use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use sqlx::PgPool;
use std::error::Error;

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow, Debug)]
struct PostgresUser {
    email: String,
    password_hash: String,
    requires_2fa: bool,
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let email = user.email.as_ref();
        let password_hash = compute_password_hash(user.password.as_ref())
            .await
            .expect("Could not compute password hash");
        let requires_2fa = user.requires_2fa;
        let _ = sqlx::query(
            r#"INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)"#
        )
        .bind(email)
        .bind(password_hash)
        .bind(requires_2fa)
        .execute(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn delete_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let _ = sqlx::query(r#"DELETE FROM users WHERE email = $1"#)
            .bind(user.email.as_ref())
            .execute(&self.pool)
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let result = sqlx::query_as::<_, PostgresUser>(
            r#"SELECT * FROM users WHERE email = $1"#,
        )
        .bind(email.as_ref())
        .fetch_one(&self.pool)
        .await
        .map_err(|_| UserStoreError::UserNotFound)?;

        let password =
            Password::parse(result.password_hash).map_err(|_| UserStoreError::UnexpectedError)?;
        let email = Email::parse(result.email).map_err(|_| UserStoreError::UnexpectedError)?;

        Ok(User::new(email, password, result.requires_2fa))
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        let password_candidate = password.as_ref();

        verify_password_hash(user.password.as_ref(), password_candidate)
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)?;

        Ok(())
    }
}

// Helper function to verify if a given password matches an expected hash.

async fn verify_password_hash(
    expected_password_hash: &str,
    password_candidate: &str,
) -> Result<(), Box<dyn Error>> {
    let expected_password_hash = expected_password_hash.to_string();
    let password_candidate = password_candidate.to_string();

    tokio::task::spawn_blocking(move || {
        let expected_password_hash = PasswordHash::new(&expected_password_hash)?;
        Argon2::default().verify_password(password_candidate.as_bytes(), &expected_password_hash)
    })
    .await??;

    Ok(())
}

// Helper function to hash passwords before persisting them in the database.
async fn compute_password_hash(password: &str) -> Result<String, Box<dyn Error>> {
    let password = password.to_string();
    let res = tokio::task::spawn_blocking(move || {
        let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None)?,
        );

        match argon2.hash_password(password.as_bytes(), &salt) {
            Ok(pass) => Ok(pass.to_string()),
            Err(e) => Err(e),
        }
    })
    .await??;

    Ok(res)
}
