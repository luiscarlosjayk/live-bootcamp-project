use auth_service::{
    app_state::{AppState, BannedTokenStoreType, EmailClientType, TwoFACodeStoreType},
    domain::path::Paths,
    services::{
        hashmap_two_fa_code_store::HashmapTwoFACodeStore, hashmap_user_store::HashmapUserStore,
        hashset_banned_token_store::HashsetBannedTokenStore, mock_email_client::MockEmailClient,
    },
    utils::constants::{self, test},
    Application,
};
use reqwest::cookie::Jar;
use std::sync::Arc;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub banned_token_store: BannedTokenStoreType,
    pub http_client: reqwest::Client,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub email_client: EmailClientType,
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

        let user_store = Arc::new(tokio::sync::RwLock::new(HashmapUserStore::default()));
        let banned_token_store =
            Arc::new(tokio::sync::RwLock::new(HashsetBannedTokenStore::default()));
        let two_fa_code_store =
            Arc::new(tokio::sync::RwLock::new(HashmapTwoFACodeStore::default()));
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

    pub async fn post_verify_2fa(&self) -> reqwest::Response {
        self.http_client
            .post(format!("{}{}", &self.address, Paths::Verify2FA.as_str()))
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
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}
