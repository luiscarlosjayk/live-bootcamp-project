use app_state::AppState;
use axum::{routing::post, serve::Serve, Router};
use std::error::Error;
use tower_http::services::ServeDir;

pub mod app_state;
pub mod domain;
pub mod routes;
pub mod services;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<Router, Router>,
    // address is exposed as a public field
    // so we have access to it in tests.
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        // Move the Router definition from `main.rs` to here.
        // Also, remove the `hello` route.
        // We don't need it at this point!
        let router = Router::new()
            .route(domain::path::Paths::Signup.as_str(), post(routes::signup))
            .route(domain::path::Paths::Login.as_str(), post(routes::login))
            .route(domain::path::Paths::Logout.as_str(), post(routes::logout))
            .route(
                domain::path::Paths::Verify2FA.as_str(),
                post(routes::verify_2fa),
            )
            .route(
                domain::path::Paths::VerifyToken.as_str(),
                post(routes::verify_token),
            )
            .nest_service(domain::path::Paths::Root.as_str(), ServeDir::new("assets"))
            .with_state(app_state);

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
