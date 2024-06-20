use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email},
    utils::auth,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Verify2FARequest {
    pub email: Secret<String>,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}

#[tracing::instrument(name = "Verify 2FA Route Handler", skip_all)]
pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = match Email::parse(request.email) {
        Ok(email) => email,
        Err(_) => {
            return (jar, Err(AuthAPIError::InvalidCredentials));
        }
    };

    let (login_attempt_id_result, two_fa_code_result) =
        match state.two_fa_code_store.read().await.get_code(&email).await {
            Ok(tuple) => tuple,
            Err(_) => {
                return (jar, Err(AuthAPIError::IncorrectCredentials));
            }
        };

    let login_attempt_id = request.login_attempt_id;

    if login_attempt_id != *login_attempt_id_result.as_ref().expose_secret() {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let two_fa_code = request.two_fa_code;

    if two_fa_code != *two_fa_code_result.as_ref().expose_secret() {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let auth_cookie = match auth::generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e))),
    };

    let jar = jar.add(auth_cookie);

    if let Err(e) = state
        .two_fa_code_store
        .write()
        .await
        .remove_code(&email)
        .await
    {
        return (jar, Err(AuthAPIError::UnexpectedError(e.into())));
    }

    (jar, Ok(StatusCode::OK.into_response()))
}
