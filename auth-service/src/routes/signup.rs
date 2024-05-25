use std::collections::HashMap;

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, User},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

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

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    // Create a new `User` instance using data in the `request`
    let email = request.email;
    let password = request.password;
    let requires_2fa = request.requires_2fa;
    let recaptcha_response = request.recaptcha;
    let recaptcha_secret = state.recaptcha_secret;
    let mut params = HashMap::new();
    let mut headers = reqwest::header::HeaderMap::new();

    params.insert("secret", &recaptcha_secret);
    params.insert("response", &recaptcha_response);

    // headers.insert(reqwest::header::HOST, "127.0.0.1".parse().unwrap());

    let api_client = reqwest::Client::builder().build().unwrap();
    let recaptcha_verify_url = "https://www.google.com/recaptcha/api/siteverify".to_string();
    let data = format!(
        "secret={}&response={}",
        recaptcha_secret, recaptcha_response
    );
    let content_length = data.len();

    headers.insert(
        reqwest::header::CONTENT_LENGTH,
        content_length.to_string().parse().unwrap(),
    );
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        "application/x-www-form-urlencoded".parse().unwrap(),
    );

    let recaptcha_verify_response = api_client
        .post(&recaptcha_verify_url)
        .headers(headers)
        .form(&params)
        .send()
        .await;

    if recaptcha_verify_response.is_err() {
        return Err(AuthAPIError::InvalidRecaptcha);
    } else if let Ok(r) = recaptcha_verify_response {
        let recaptcha_verify_response_body: RecaptchaResponse =
            r.json::<RecaptchaResponse>().await.unwrap();

        if !recaptcha_verify_response_body.success {
            return Err(AuthAPIError::InvalidRecaptcha);
        } else {
            println!(
                "Recaptcha verification failed: {:?}",
                recaptcha_verify_response_body.error_codes
            );
        }
    }

    if !email.contains('@') || email.is_empty() || password.len() < 8 {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let user = User::new(email, password, requires_2fa);

    let mut user_store = state.user_store.write().await;

    // If user already exists then return 409
    if user_store
        .get_user(user.email.as_ref().to_string())
        .await
        .is_ok()
    {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    let add_user_result = user_store.add_user(user).await;
    if add_user_result.is_err() {
        return Err(AuthAPIError::UnexpectedError);
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}
