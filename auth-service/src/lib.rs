use app_state::AppState;
use axum::{
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{delete, post},
    serve::Serve,
    Json, Router,
};
use domain::{environment::get_env, AuthAPIError};
use redis::{Client as RedisClient, RedisResult};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::error::Error;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use utils::constants;
use utils::tracing::{make_span_with_request_id, on_request, on_response};

pub mod app_state;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let droplet_ip = get_env(constants::env::DROPLET_IP_ENV_VAR);
        let base_path = get_env(constants::env::BASE_PATH_ENV_VAR);
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
            .layer(cors)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(make_span_with_request_id)
                    .on_request(on_request)
                    .on_response(on_response),
            );

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        // Create a new Application instance and return it
        Ok(Self {
            address: address.to_string(),
            server,
        })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        tracing::info!("listening on {}", &self.address);
        self.server.await?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> axum::response::Response {
        log_error_chain(&self);

        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::UnexpectedError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
            }
            AuthAPIError::InvalidRecaptcha => (StatusCode::BAD_REQUEST, "Invalid captcha"),
            AuthAPIError::IncorrectCredentials => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Missing auth token"),
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid auth token"),
        };
        let body = Json(ErrorResponse {
            error: error_message.to_owned(),
        });
        (status, body).into_response()
    }
}

fn log_error_chain(e: &(dyn Error + 'static)) {
    let separator =
        "\n-----------------------------------------------------------------------------------\n";
    let mut report = format!("{}{:?}\n", separator, e);
    let mut current = e.source();
    while let Some(cause) = current {
        let str = format!("Caused by:\n\n{:?}", cause);
        report = format!("{}\n{}", report, str);
        current = cause.source();
    }
    report = format!("{}\n{}", report, separator);
    tracing::error!("{}", report);
}

pub async fn get_postgres_pool(url: &Secret<String>) -> Result<PgPool, sqlx::Error> {
    // Creates a new PostgreSQL connection pool.
    PgPoolOptions::new()
        .max_connections(5)
        .connect(url.expose_secret())
        .await
}

pub fn get_redis_client(redis_hostname: String) -> RedisResult<RedisClient> {
    let redis_url = format!("redis://{}/", redis_hostname);
    redis::Client::open(redis_url)
}
