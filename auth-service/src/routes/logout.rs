use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    domain::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

pub async fn logout(jar: CookieJar) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // Retrieve JWT cookie from the `CookieJar`
    // Return AuthAPIError::MissingToken is the cookie is not found
    let cookie = jar.get(JWT_COOKIE_NAME);

    if cookie.is_none() {
        return (jar, Err(AuthAPIError::MissingToken));
    }

    let cookie = cookie.unwrap(); // @todo: Compare with teacher's codebase if there's an easier way to extract this without unwrap
    let token = cookie.value().to_string();

    // TODO: Validate JWT token by calling `validate_token` from the auth service.
    // If the token is valid you can ignore the returned claims for now.
    // Return AuthAPIError::InvalidToken is validation fails.
    let validate_token_response = validate_token(&token).await;

    if let Err(_err) = validate_token_response {
        return (jar, Err(AuthAPIError::InvalidToken));
    }

    // Removes cookie
    let jar = jar.remove(axum_extra::extract::cookie::Cookie::from(JWT_COOKIE_NAME));

    (jar, Ok(StatusCode::OK))
}
