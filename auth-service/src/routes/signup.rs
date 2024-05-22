use crate::{
    app_state::AppState,
    domain::{AuthAPIError, User},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    // Create a new `User` instance using data in the `request`
    let email = request.email;
    let password = request.password;
    let requires_2fa = request.requires_2fa;

    if !email.contains('@') || email.is_empty() || password.len() < 8 {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let user = User::new(email, password, requires_2fa);

    let mut user_store = state.user_store.write().await;

    // If user already exists then return 409
    if user_store.get_user(user.email.clone()).is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    let add_user_result = user_store.add_user(user);
    if add_user_result.is_err() {
        return Err(AuthAPIError::UnexpectedError);
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
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
}
