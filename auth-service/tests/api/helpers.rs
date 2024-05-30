use auth_service::app_state::{AppState, BannedTokenStoreType};
use auth_service::domain::path::Paths;
use auth_service::services::banned_token_store::HashsetBannedTokenStore;
use auth_service::services::hashmap_user_store::HashmapUserStore;
use auth_service::utils::constants;
use auth_service::Application;
use reqwest::cookie::Jar;
use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
    pub cookie_jar: Arc<Jar>,
    pub banned_token_store: BannedTokenStoreType,
}

impl TestApp {
    pub async fn new() -> Self {
        /*
         * How to test recaptcha:
         * https://developers.google.com/recaptcha/docs/faq#id-like-to-run-automated-tests-with-recaptcha.-what-should-i-do
         */
        env::set_var(
            constants::env::RECAPTCHA_SECRET_ENV_VAR,
            "6LeIxAcTAAAAAGG-vFI1TnRWxMZNFuojJ4WifJWe",
        );
        env::set_var(constants::env::DROPLET_IP_ENV_VAR, "127.0.0.1");
        env::set_var(constants::env::JWT_SECRET_ENV_VAR, "foobar");
        env::set_var(constants::env::BASE_PATH_ENV_VAR, "http://localhost");

        let user_store = Arc::new(tokio::sync::RwLock::new(HashmapUserStore {
            users: HashMap::default(),
        }));
        let banned_token_store = Arc::new(tokio::sync::RwLock::new(HashsetBannedTokenStore {
            tokens: HashSet::default(),
        }));
        let app_state = AppState::new(user_store, banned_token_store.clone());
        let app = Application::build(app_state, constants::test::APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task
        // to avoid blocking the main test thread.
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        // Create new `TestApp` instance and return it
        Self {
            address,
            http_client,
            cookie_jar,
            banned_token_store: banned_token_store.clone(),
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}{}", &self.address, Paths::Root.as_str()))
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
            .post(&format!("{}{}", &self.address, Paths::Logout.as_str()))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}{}", &self.address, Paths::Verify2FA.as_str()))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}{}", &self.address, Paths::VerifyToken.as_str()))
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
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}
