use auth_service::app_state::AppState;
use auth_service::services::mock_email_client::MockEmailClient;
use auth_service::services::postgres_user_store::PostgresUserStore;
use auth_service::services::{
    data_stores::HashmapTwoFACodeStore, data_stores::HashsetBannedTokenStore,
};
use auth_service::utils::constants::{prod, DATABASE_URL};
use auth_service::{get_postgres_pool, Application};
use dotenvy::dotenv;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let pg_pool = configure_postgresql().await;

    let _res = sqlx::query("SELECT * FROM users").fetch_all(&pg_pool).await;

    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
    let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
    let email_client = Arc::new(MockEmailClient);

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
