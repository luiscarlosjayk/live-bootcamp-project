use crate::{
    app_state::AppState,
    domain::AuthAPIError,
    utils::{auth, constants::JWT_COOKIE_NAME},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use secrecy::Secret;

#[tracing::instrument(name = "Logout Route Handler", skip_all)]
pub async fn logout(
    jar: CookieJar,
    State(state): State<AppState>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let cookie = jar.get(JWT_COOKIE_NAME);

    if cookie.is_none() {
        return (jar, Err(AuthAPIError::MissingToken));
    }

    let cookie = cookie.unwrap(); // @todo: Compare with teacher's codebase if there's an easier way to extract this without unwrap
    let token = cookie.value().to_string();
    let banned_token_store = state.banned_token_store.clone();

    let response = auth::validate_token(&token, banned_token_store).await;

    if let Err(_err) = response {
        return (jar, Err(AuthAPIError::InvalidToken));
    }

    // Removes cookie
    let jar = jar.remove(axum_extra::extract::cookie::Cookie::from(JWT_COOKIE_NAME));

    // Add token to banned token store
    let mut banned_token_store = state.banned_token_store.write().await;
    if let Err(e) = banned_token_store.add_token(Secret::new(token)).await {
        return (jar, Err(AuthAPIError::UnexpectedError(e.into())));
    }

    (jar, Ok(StatusCode::OK))
}
