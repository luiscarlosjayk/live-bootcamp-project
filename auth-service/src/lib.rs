use core::fmt;
use std::error::Error;

use axum::{response::IntoResponse, routing::post, serve::Serve, Router};
use reqwest::StatusCode;
use tower_http::services::ServeDir;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<Router, Router>,
    // address is exposed as a public field
    // so we have access to it in tests.
    pub address: String,
}

pub enum Paths {
    Root,
    Signup,
    Login,
    Verify2FA,
    VerifyToken,
}

impl Paths {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "/",
            Self::Signup => "/signup",
            Self::Login => "/login",
            Self::Verify2FA => "/verify-2fa",
            Self::VerifyToken => "/verify-token",
        }
    }
}

impl fmt::Display for Paths {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            Self::Root => "/",
            Self::Signup => "/signup",
            Self::Login => "/login",
            Self::Verify2FA => "/verify-2fa",
            Self::VerifyToken => "/verify-token",
        };
        write!(f, "{}", output)
    }
}

impl Application {
    pub async fn build(address: &str) -> Result<Self, Box<dyn Error>> {
        // Move the Router definition from `main.rs` to here.
        // Also, remove the `hello` route.
        // We don't need it at this point!
        let router = Router::new()
            .route(Paths::Signup.as_str(), post(signup))
            .route(Paths::Login.as_str(), post(login))
            .route(Paths::Verify2FA.as_str(), post(verify_2fa))
            .route(Paths::VerifyToken.as_str(), post(verify_token))
            .nest_service(Paths::Root.as_str(), ServeDir::new("assets"));

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        // Create a new Application instance and return it
        Ok(Self { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}

// Example route handler.
// For now we will simply return a 200 (OK) status code.
async fn signup() -> impl IntoResponse {
    StatusCode::OK.into_response()
}

async fn login() -> impl IntoResponse {
    StatusCode::OK.into_response()
}

async fn verify_2fa() -> impl IntoResponse {
    StatusCode::OK.into_response()
}

async fn verify_token() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
