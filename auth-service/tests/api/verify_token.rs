use crate::helpers::{get_random_email, TestApp};
use auth_service::{domain::Email, utils::auth};

#[tokio::test]
async fn verify_token_should_return_422_if_malformed() {
    let app = TestApp::new().await;
    let body = serde_json::json!({
        "invalid": "invalid",
    });
    let response = app.post_verify_token(&body).await;

    assert_eq!(response.status().as_u16(), 422);
}

#[tokio::test]
async fn verify_token_should_return_200_valid_token() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let random_email = Email::parse(random_email).unwrap();
    let token = auth::generate_auth_token(&random_email).unwrap();
    let body = serde_json::json!({
        "token": token,
    });
    let response = app.post_verify_token(&body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn verify_token_should_return_401_if_invalid_token() {
    let app = TestApp::new().await;
    let body = serde_json::json!({
        "token": "invalid",
    });
    let response = app.post_verify_token(&body).await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn verify_token_should_return_401_if_banned_token() {
    let app = TestApp::new().await;
    let random_email = Email::parse(get_random_email()).unwrap();
    let token = auth::generate_auth_token(&random_email).unwrap();
    {
        let mut banned_token_store = app.banned_token_store.write().await;

        let response = banned_token_store.add_token(token.clone()).await;
        assert!(response.is_ok());
    }

    let body = serde_json::json!({
        "token": token,
    });
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status().as_u16(), 401);
}
