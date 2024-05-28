use app_state::AppState;
use axum::{
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{delete, post},
    serve::Serve,
    Json, Router,
};
use domain::{environment::get_env, AuthAPIError};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tower_http::{cors::CorsLayer, services::ServeDir};
use utils::constants;

pub mod app_state;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<Router, Router>,
    // address is exposed as a public field
    // so we have access to it in tests.
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let droplet_ip = get_env(constants::env::DROPLET_IP_ENV_VAR.to_string());
        let base_path = get_env(constants::env::BASE_PATH_ENV_VAR.to_string());
        let allowed_origins = [
            "http://localhost:8000".parse()?,
            format!("https://{}:8000", droplet_ip.as_str()).parse()?,
            format!("{}/app", base_path.as_str()).parse()?,
        ];

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_credentials(true)
            .allow_origin(allowed_origins);

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
            .route(
                format!("{}/:email", domain::path::Paths::Users.as_str()).as_str(),
                delete(routes::delete),
            )
            .nest_service(domain::path::Paths::Root.as_str(), ServeDir::new("assets"))
            .with_state(app_state)
            .layer(cors);

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

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::UnexpectedError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
            }
            AuthAPIError::InvalidRecaptcha => (StatusCode::BAD_REQUEST, "Invalid captcha"),
            AuthAPIError::IncorrectCredentials => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Missing token"),
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
        };
        let body = Json(ErrorResponse {
            error: error_message.to_owned(),
        });
        (status, body).into_response()
    }
}
