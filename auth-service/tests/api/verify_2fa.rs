use auth_service::{domain::Email, utils::constants::JWT_COOKIE_NAME};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_200_if_correct_code() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();
    let email = Email::parse(random_email.clone()).expect("Could not parse random_email to Email");
    let password = "abcDEF123".to_string();
    // Signup
    let body = serde_json::json!({
        "email": &random_email,
        "password": &password,
        "requires2FA": true,
        "recaptcha": "recaptcha",
    });
    let response = app.post_signup(&body).await;

    assert_eq!(response.status().as_u16(), 201);

    // Login
    let body = serde_json::json!({
        "email": &random_email,
        "password": &password,
    });

    let response = app.post_login(&body).await;

    assert_eq!(response.status().as_u16(), 206);

    let (login_attempt_id, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(email.clone())
        .await
        .unwrap();

    // Verify 2FA
    let body = serde_json::json!({
        "email": &random_email,
        "2FACode": two_fa_code.as_ref(),
        "loginAttemptId": login_attempt_id.as_ref(),
    });

    let response = app.post_verify_2fa(&body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    // Clean up database
    app.clean_up().await;
    
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;

    let malformed_request = serde_json::json!({
        "email": "test@test.com",
        "2FACode": "1234",
    });

    let response = app.post_verify_2fa(&malformed_request).await;
    assert_eq!(response.status().as_u16(), 422);

    // Clean up database
    app.clean_up().await;
    
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;

    let invalid_request = serde_json::json!({
        "email": "not_an_email",
        "2FACode": "1234",
        "loginAttemptId": "1234",
    });

    let response = app.post_verify_2fa(&invalid_request).await;
    assert_eq!(response.status().as_u16(), 400);

    // Clean up database
    app.clean_up().await;
    
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();
    let email = Email::parse(random_email.clone()).expect("Could not parse random_email to Email");
    let password = "abcDEF123".to_string();
    // Signup
    let body = serde_json::json!({
        "email": &random_email,
        "password": &password,
        "requires2FA": true,
        "recaptcha": "recaptcha",
    });
    let response = app.post_signup(&body).await;

    assert_eq!(response.status().as_u16(), 201);

    // Login
    let body = serde_json::json!({
        "email": &random_email,
        "password": &password,
    });

    let response = app.post_login(&body).await;

    assert_eq!(response.status().as_u16(), 206);

    let (login_attempt_id, _) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(email.clone())
        .await
        .unwrap();

    // Verify 2FA
    let body = serde_json::json!({
        "email": &random_email,
        "2FACode": "invalid_two_fa",
        "loginAttemptId": login_attempt_id.as_ref(),
    });

    let response = app.post_verify_2fa(&body).await;

    assert_eq!(response.status().as_u16(), 401);

    // Clean up database
    app.clean_up().await;
    
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();
    let email = Email::parse(random_email.clone()).expect("Could not parse random_email to Email");
    let password = "abcDEF123".to_string();
    // Signup
    let body = serde_json::json!({
        "email": &random_email,
        "password": &password,
        "requires2FA": true,
        "recaptcha": "recaptcha",
    });
    let response = app.post_signup(&body).await;

    assert_eq!(response.status().as_u16(), 201);

    // Login
    let body = serde_json::json!({
        "email": &random_email,
        "password": &password,
    });

    let response = app.post_login(&body).await;

    assert_eq!(response.status().as_u16(), 206);

    let (login_attempt_id, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(email.clone())
        .await
        .unwrap();

    // Second Login
    let body = serde_json::json!({
        "email": &random_email,
        "password": &password,
    });

    let response = app.post_login(&body).await;

    assert_eq!(response.status().as_u16(), 206);

    // Verify 2FA
    let body = serde_json::json!({
        "email": &random_email,
        "2FACode": two_fa_code.as_ref(),
        "loginAttemptId": login_attempt_id.as_ref(),
    });

    let response = app.post_verify_2fa(&body).await;

    assert_eq!(response.status().as_u16(), 401);

    // Clean up database
    app.clean_up().await;
    
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();
    let email = Email::parse(random_email.clone()).expect("Could not parse random_email to Email");
    let password = "abcDEF123".to_string();
    // Signup
    let body = serde_json::json!({
        "email": &random_email,
        "password": &password,
        "requires2FA": true,
        "recaptcha": "recaptcha",
    });
    let response = app.post_signup(&body).await;

    assert_eq!(response.status().as_u16(), 201);

    // Login
    let body = serde_json::json!({
        "email": &random_email,
        "password": &password,
    });

    let response = app.post_login(&body).await;

    assert_eq!(response.status().as_u16(), 206);

    let (login_attempt_id, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(email.clone())
        .await
        .unwrap();

    // Verify 2FA
    let body = serde_json::json!({
        "email": &random_email,
        "2FACode": two_fa_code.as_ref(),
        "loginAttemptId": login_attempt_id.as_ref(),
    });

    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status().as_u16(), 200);

    let response = app.post_verify_2fa(&body).await;
    dbg!(&response.status().as_u16());

    assert_eq!(response.status().as_u16(), 401);

    // Clean up database
    app.clean_up().await;
    
}
