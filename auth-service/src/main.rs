use auth_service::app_state::AppState;
use auth_service::domain::Email;
use auth_service::services::postgres_user_store::PostgresUserStore;
use auth_service::services::{
    aws_ses_email_client::SESEmailClient, data_stores::RedisBannedTokenStore,
    data_stores::RedisTwoFACodeStore,
};
use auth_service::utils::constants::{prod, DATABASE_URL, REDIS_HOST_NAME};
use auth_service::utils::tracing::init_tracing;
use auth_service::{get_postgres_pool, get_redis_client, Application};
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use dotenvy::dotenv;
use secrecy::Secret;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    dotenv().ok();
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing");

    let pg_pool = configure_postgresql().await;
    let redis_connection = Arc::new(RwLock::new(configure_redis()));

    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(
        redis_connection.clone(),
    )));
    let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_connection)));
    let email_client = Arc::new(configure_ses_email_client().await);

    let app_state = AppState::new(
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client,
    );

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool.
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

async fn configure_aws_config() -> aws_config::SdkConfig {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");

    aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await
}

async fn configure_ses_email_client() -> SESEmailClient {
    let sdk_config = configure_aws_config().await;
    let sender = Email::parse(Secret::new(
        auth_service::utils::constants::EMAIL_SENDER.to_owned(),
    ))
    .expect("Failed to create Email from EMAIL_SENDER env");

    SESEmailClient::new(sender, &sdk_config)
}
