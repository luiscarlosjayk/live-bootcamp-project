use auth_service::{Application, Paths};

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let app = Application::build("127.0.0.1:0")
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

    pub async fn signup(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}{}", &self.address, Paths::Signup.as_str()))
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
}