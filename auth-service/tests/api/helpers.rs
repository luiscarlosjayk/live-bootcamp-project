use auth_service::app_state::AppState;
use auth_service::domain::path::Paths;
use auth_service::services::hashmap_user_store::HashmapUserStore;
use auth_service::Application;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        /*
         * How to test recaptcha:
         * https://developers.google.com/recaptcha/docs/faq#id-like-to-run-automated-tests-with-recaptcha.-what-should-i-do
         */
        env::set_var(
            "RECAPTCHA_SECRET",
            "6LeIxAcTAAAAAGG-vFI1TnRWxMZNFuojJ4WifJWe",
        );
        let user_store = Arc::new(tokio::sync::RwLock::new(HashmapUserStore {
            users: HashMap::default(),
        }));
        let app_state = AppState::new(user_store);
        let app = Application::build(app_state, "127.0.0.1:0")
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task
        // to avoid blocking the main test thread.
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let http_client = reqwest::Client::new(); // Create a Reqwest http client instance

        // Create new `TestApp` instance and return it
        Self {
            address,
            http_client,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}{}", &self.address, Paths::Root.as_str()))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn signup<Body>(&self, body: &Body) -> reqwest::Response
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

    pub async fn login(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}{}", &self.address, Paths::Login.as_str()))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}{}", &self.address, Paths::Logout.as_str()))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn verify_2fa(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}{}", &self.address, Paths::Verify2FA.as_str()))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn verify_token(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}{}", &self.address, Paths::VerifyToken.as_str()))
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
