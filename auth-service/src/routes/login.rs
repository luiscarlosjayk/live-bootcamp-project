use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, Password},
    utils::auth::generate_auth_cookie,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct LoginResponse {
    pub message: String,
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = Email::parse(request.email).ok();
    let password = Password::parse(request.password).ok();

    if email.is_none() || password.is_none() {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    }

    let email = email.unwrap();
    let password = password.unwrap();

    let user_store = state.user_store.read().await;

    let validate_response = user_store.validate_user(&email, &password).await.ok();

    let user = user_store.get_user(&email).await.ok();

    if validate_response.is_none() || user.is_none() {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let response = Json(LoginResponse {
        message: "Logged in successfully".to_string(),
    });

    // Ok((StatusCode::OK, response))

    let auth_cookie = generate_auth_cookie(&email);

    if auth_cookie.is_err() {
        return (jar, Err(AuthAPIError::UnexpectedError));
    }

    let updated_jar = jar.add(auth_cookie.ok().unwrap());

    (updated_jar, Ok((StatusCode::OK, response)))
}
