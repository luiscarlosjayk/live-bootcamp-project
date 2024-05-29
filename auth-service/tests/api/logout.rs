use crate::helpers::{get_random_email, TestApp};
use auth_service::{
    domain::environment::get_env,
    utils::{
        auth::{Claims, TOKEN_TTL_SECONDS},
        constants::{env::JWT_SECRET_ENV_VAR, JWT_COOKIE_NAME},
    },
};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use reqwest::Url;

#[tokio::test]
async fn logout_should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn logout_should_return_401_if_invalid_jwt() {
    let app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn logout_should_return_200_if_valid_jwt_cookie() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let claims = Claims {
        sub: random_email,
        exp: Utc::now()
            .checked_add_signed(
                chrono::Duration::try_seconds(TOKEN_TTL_SECONDS).expect("Valid duration"),
            )
            .expect("valid timestamp")
            .timestamp() as usize,
    };
    let jwt_secret = get_env(JWT_SECRET_ENV_VAR);
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .expect("Failed to encode JWT");

    app.cookie_jar.add_cookie_str(
        &format!(
            "{}={token}; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn logout_should_return_400_if_logout_called_twice_in_a_row() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let claims = Claims {
        sub: random_email,
        exp: Utc::now()
            .checked_add_signed(
                chrono::Duration::try_seconds(TOKEN_TTL_SECONDS).expect("Valid duration"),
            )
            .expect("valid timestamp")
            .timestamp() as usize,
    };
    let jwt_secret = get_env(JWT_SECRET_ENV_VAR.to_string());
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .expect("Failed to encode JWT");

    app.cookie_jar.add_cookie_str(
        &format!(
            "{}={token}; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 200);

    let response = app.post_logout().await; // Second time!
    assert_eq!(response.status().as_u16(), 400);
}
