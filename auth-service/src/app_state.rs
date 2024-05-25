use crate::domain::data_stores::UserStore;
use std::sync::Arc;
use tokio::sync::RwLock;

// Using a type alias to improve readability!
pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub recaptcha_secret: String,
}

impl AppState {
    pub fn new(user_store: UserStoreType, recaptcha_secret: String) -> Self {
        Self {
            user_store,
            recaptcha_secret,
        }
    }
}
