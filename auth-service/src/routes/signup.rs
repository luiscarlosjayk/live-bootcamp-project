use crate::{
    app_state::AppState,
    domain::{environment::get_env, AuthAPIError, Email, Password, User},
    utils::constants::env::RECAPTCHA_SECRET_ENV_VAR,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct RecaptchaResponse {
    pub success: bool,
    pub challenge_ts: Option<String>,
    pub hostname: Option<String>,
    pub score: Option<f32>,
    pub action: Option<String>,
    #[serde(rename = "error-codes")]
    pub error_codes: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SignupResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
    pub recaptcha: String,
}

async fn validate_recaptcha(token: String) -> bool {
    let recaptcha_secret = get_env(RECAPTCHA_SECRET_ENV_VAR);
    let mut params = HashMap::new();
    let mut headers = reqwest::header::HeaderMap::new();

    params.insert("secret", &recaptcha_secret);
    params.insert("response", &token);

    let api_client = reqwest::Client::builder().build().unwrap();
    let recaptcha_verify_url = "https://www.google.com/recaptcha/api/siteverify".to_string();
    let data = format!("secret={}&response={}", recaptcha_secret, token);
    let content_length = data.len();

    headers.insert(
        reqwest::header::CONTENT_LENGTH,
        content_length.to_string().parse().unwrap(),
    );
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        "application/x-www-form-urlencoded".parse().unwrap(),
    );

    let recaptcha_response = api_client
        .post(&recaptcha_verify_url)
        .headers(headers)
        .form(&params)
        .send()
        .await;

    recaptcha_response.is_ok()
}

#[tracing::instrument(name = "Signup", skip_all)]
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    // Create a new `User` instance using data in the `request`
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password =
        Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let requires_2fa = request.requires_2fa;

    let is_recaptcha_valid = validate_recaptcha(request.recaptcha).await;

    if !is_recaptcha_valid {
        return Err(AuthAPIError::InvalidRecaptcha);
    }

    let mut user_store = state.user_store.write().await;
    let user = User::new(email, password, requires_2fa);

    // If user already exists then return 409
    if user_store.get_user(&user.email).await.is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    if let Err(e) = user_store.add_user(user).await {
        return Err(AuthAPIError::UnexpectedError(e.into()));
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}
