use auth_service::app_state::AppState;
use auth_service::services::hashmap_user_store::HashmapUserStore;
use auth_service::Application;
use dotenvy::dotenv;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let environment = env::var("ENVIRONMENT").expect("Missing ENVIRONMENT env.");
    println!("Starting environment: {environment}");
    if environment == *"local" {
        dotenv().expect("Couldn't load the .env file in local environment.");
    }
    let recaptcha_secret = env::var("RECAPTCHA_SECRET").expect("Missing RECAPTCHA_SECRET");
    let user_store = Arc::new(RwLock::new(HashmapUserStore {
        users: HashMap::default(),
    }));
    let app_state = AppState::new(user_store, recaptcha_secret);

    let app = Application::build(app_state, "0.0.0.0:3000")
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
