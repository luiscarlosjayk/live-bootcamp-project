use auth_service::app_state::AppState;
use auth_service::services::{
    banned_token_store::HashsetBannedTokenStore, hashmap_user_store::HashmapUserStore,
};
use auth_service::utils::constants::prod;
use auth_service::Application;
use dotenvy::dotenv;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let user_store = Arc::new(RwLock::new(HashmapUserStore {
        users: HashMap::default(),
    }));
    let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore {
        tokens: HashSet::default(),
    }));
    let app_state = AppState::new(user_store, banned_token_store);

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
