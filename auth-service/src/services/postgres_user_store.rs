use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    Email, Password, User,
};
use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use color_eyre::eyre::{eyre, Context, Result};
use sqlx::{FromRow, PgPool};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow, Debug)]
struct PostgresUser {
    email: String,
    password_hash: String,
    requires_2fa: bool,
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let email = user.email.as_ref();
        let password_hash = compute_password_hash(user.password.as_ref().to_string())
            .await
            .map_err(UserStoreError::UnexpectedError)?;
        let requires_2fa = user.requires_2fa;

        sqlx::query!(
            r#"INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)"#,
            email,
            password_hash,
            requires_2fa
        )
        .execute(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    #[tracing::instrument(name = "Deleting user from PostgreSQL", skip_all)]
    async fn delete_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let _ = sqlx::query!(r#"DELETE FROM users WHERE email = $1"#, user.email.as_ref())
            .execute(&self.pool)
            .await
            .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let result = sqlx::query_as!(
            PostgresUser,
            r#"SELECT * FROM users WHERE email = $1"#,
            email.as_ref()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| UserStoreError::UserNotFound)?;

        let password = Password::parse(result.password_hash)
            .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?;
        let email =
            Email::parse(result.email).map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?;

        Ok(User::new(email, password, result.requires_2fa))
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        let password_candidate = password.as_ref().to_string();

        verify_password_hash(user.password.as_ref().to_string(), password_candidate)
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)?;

        Ok(())
    }
}

// Helper function to verify if a given password matches an expected hash.

#[tracing::instrument(name = "Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<()> {
    let current_span: tracing::Span = tracing::Span::current();

    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let expected_password_hash: PasswordHash<'_> =
                PasswordHash::new(&expected_password_hash)?;

            Argon2::default()
                .verify_password(password_candidate.as_bytes(), &expected_password_hash)
                .wrap_err("Failed to verify password hash")
        })
    })
    .await;

    result?
}

// Helper function to hash passwords before persisting them in the database.
#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: String) -> Result<String> {
    let current_span: tracing::Span = tracing::Span::current(); // New!

    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

            Ok(password_hash)
        })
    })
    .await;

    result?
}
