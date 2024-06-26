use auth_service::{
    app_state::{AppState, BannedTokenStoreType, EmailClientType, TwoFACodeStoreType},
    domain::path::Paths,
    get_postgres_pool,
    services::{
        data_stores::{RedisBannedTokenStore, RedisTwoFACodeStore},
        mock_email_client::MockEmailClient,
        postgres_user_store::PostgresUserStore,
    },
    utils::constants::{self, test},
    Application,
};
use redis::{Client as RedisClient, RedisResult};
use reqwest::cookie::Jar;
use secrecy::{ExposeSecret, Secret};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub banned_token_store: BannedTokenStoreType,
    pub http_client: reqwest::Client,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub email_client: EmailClientType,
    pub database_name: String,
    pub clean_up_called: bool,
}

impl TestApp {
    pub async fn new() -> Self {
        /*
         * How to test recaptcha:
         * https://developers.google.com/recaptcha/docs/faq#id-like-to-run-automated-tests-with-recaptcha.-what-should-i-do
         */
        std::env::set_var(
            constants::env::RECAPTCHA_SECRET_ENV_VAR,
            "6LeIxAcTAAAAAGG-vFI1TnRWxMZNFuojJ4WifJWe",
        );
        std::env::set_var(constants::env::DROPLET_IP_ENV_VAR, "127.0.0.1");
        std::env::set_var(constants::env::JWT_SECRET_ENV_VAR, "foobar");
        std::env::set_var(constants::env::BASE_PATH_ENV_VAR, "http://localhost");

        // We are creating a new database for each test case, and we need to ensure each database has a unique name!
        let database_name = Uuid::new_v4().to_string();
        let pg_pool = configure_postgresql(&database_name).await;
        let redis_connection = Arc::new(tokio::sync::RwLock::new(configure_redis()));

        let user_store = Arc::new(tokio::sync::RwLock::new(PostgresUserStore::new(pg_pool)));
        let banned_token_store = Arc::new(tokio::sync::RwLock::new(RedisBannedTokenStore::new(
            redis_connection.clone(),
        )));
        let two_fa_code_store = Arc::new(tokio::sync::RwLock::new(RedisTwoFACodeStore::new(
            redis_connection,
        )));
        let email_client = Arc::new(MockEmailClient);

        let app_state = AppState::new(
            user_store,
            banned_token_store.clone(),
            two_fa_code_store.clone(),
            email_client.clone(),
        );

        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        Self {
            address,
            cookie_jar,
            banned_token_store,
            http_client,
            two_fa_code_store,
            email_client,
            database_name,
            clean_up_called: false,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}{}", &self.address, Paths::Signup.as_str()))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}{}", &self.address, Paths::Login.as_str()))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(format!("{}{}", &self.address, Paths::Logout.as_str()))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}{}", &self.address, Paths::Verify2FA.as_str()))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}{}", &self.address, Paths::VerifyToken.as_str()))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn delete_user(&self, email: String) -> reqwest::Response {
        self.http_client
            .delete(&format!(
                "{}{}/{}",
                &self.address,
                Paths::Users.as_str(),
                email
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn clean_up(&mut self) {
        let database_name = &self.database_name;
        delete_database(database_name).await;
        self.clean_up_called = true;
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.clean_up_called {
            panic!("AppState was not cleaned up!");
        }
    }
}

fn configure_redis() -> redis::Connection {
    let redis_hostname = constants::REDIS_HOST_NAME.to_owned();

    get_redis_client(redis_hostname)
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

fn get_redis_client(redis_hostname: String) -> RedisResult<RedisClient> {
    let redis_url = format!("redis://{}/", redis_hostname);
    redis::Client::open(redis_url)
}

async fn configure_postgresql(db_name: &str) -> PgPool {
    let postgresql_conn_url = constants::DATABASE_URL.expose_secret();

    // // We are creating a new database for each test case, and we need to ensure each database has a unique name!
    // let db_name = Uuid::new_v4().to_string();

    configure_database(&postgresql_conn_url, db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    get_postgres_pool(&Secret::new(postgresql_conn_url_with_db))
        .await
        .expect("Failed to create Postgres connection pool!")
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url = constants::DATABASE_URL.expose_secret().to_owned();
    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string.");
    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to PostgreSQL.");

    // Shutdown any active connection to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                AND pid <> pg_backend_pid();
            "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}
