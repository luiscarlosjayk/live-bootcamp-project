use std::error::Error;

use axum::{routing::post, serve::Serve, Router};
use tower_http::services::ServeDir;

pub mod routes;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<Router, Router>,
    // address is exposed as a public field
    // so we have access to it in tests.
    pub address: String,
}

impl Application {
    pub async fn build(address: &str) -> Result<Self, Box<dyn Error>> {
        // Move the Router definition from `main.rs` to here.
        // Also, remove the `hello` route.
        // We don't need it at this point!
        let router = Router::new()
            .route(routes::Paths::Signup.as_str(), post(routes::signup))
            .route(routes::Paths::Login.as_str(), post(routes::login))
            .route(routes::Paths::Logout.as_str(), post(routes::logout))
            .route(routes::Paths::Verify2FA.as_str(), post(routes::verify_2fa))
            .route(
                routes::Paths::VerifyToken.as_str(),
                post(routes::verify_token),
            )
            .nest_service(routes::Paths::Root.as_str(), ServeDir::new("assets"));

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
