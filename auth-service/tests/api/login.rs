use crate::helpers::{get_random_email, TestApp};
use auth_service::{
    routes::TwoFactorAuthResponse,
    utils::constants::{
        env::{BASE_PATH_ENV_VAR, DROPLET_IP_ENV_VAR},
        JWT_COOKIE_NAME,
    },
    ErrorResponse,
};

#[tokio::test]
async fn login_should_return_422_if_malformed_credentials() {
    let mut app = TestApp::new().await;
    let invalid_inputs = [
        serde_json::json!({
            "email": get_random_email(),
        }),
        serde_json::json!({
            "password": "abcEFG123"
        }),
        serde_json::json!({
            "email": get_random_email(),
            "password": 123, // Shouldn't be a number
        }),
        serde_json::json!({
            "email": true, // Shouldn't be a bool
            "password": "abcEFG123",
        }),
    ];

    for invalid_body in invalid_inputs.iter() {
        let response = app.post_login(invalid_body).await;

        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            invalid_body
        );
    }

    // Clean up database
    app.clean_up().await;
}

#[tokio::test]
async fn login_should_return_400_if_invalid_input() {
    std::env::set_var(DROPLET_IP_ENV_VAR, "127.0.0.1");
    std::env::set_var(BASE_PATH_ENV_VAR, "localhost");
    let mut app = TestApp::new().await;
    let invalid_inputs = [
        serde_json::json!({
            "email": get_random_email(),
            "password": "abcdef123", // Should container uppercase
        }),
        serde_json::json!({
            "email": get_random_email(),
            "password": "abcdDEFG", // Should contain number
        }),
        serde_json::json!({
            "email": get_random_email(),
            "password": "ABCDEF123", // Should contain lowercase
        }),
        serde_json::json!({
            "email": get_random_email(),
            "password": "abcDEF", // Should be longer than 8 characters
        }),
        serde_json::json!({
            "email": "@test.com", // Missing username
            "password": "abcDEF123",
        }),
        serde_json::json!({
            "email": "invalid_email.com", // Missing @
            "password": "abcDEF123",
        }),
    ];

    for invalid_input in invalid_inputs.iter() {
        let response = app.post_login(invalid_input).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            invalid_input
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_string()
        )
    }

    // Clean up database
    app.clean_up().await;
}

#[tokio::test]
async fn login_should_return_401_if_incorrect_credentials() {
    let mut app = TestApp::new().await;
    let body = serde_json::json!({
        "email": "unexisting@test.com",
        "password": "abcDEF123",
    }); // This user doesn't exist

    let login_response = app.post_login(&body).await;

    assert_eq!(login_response.status().as_u16(), 401);
    assert_eq!(
        login_response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Unauthorized".to_string()
    );

    // Clean up database
    app.clean_up().await;
}

#[tokio::test]
async fn login_should_return_200_if_valid_credentials_and_2fa_disabled() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "abcDEF123",
        "requires2FA": false,
        "recaptcha": "recaptcha",
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "abcDEF123",
    });

    let response = app.post_login(&login_body).await;
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
async fn login_should_return_206_if_valid_credentials_and_2fa_enabled() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "abcDEF123",
        "requires2FA": true,
        "recaptcha": "recaptcha",
    });
    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email.clone(),
        "password": "abcDEF123",
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 206);

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_string());

    let login_attempt_id = json_body.login_attempt_id;
    assert!(!login_attempt_id.is_empty());

    // Clean up database
    app.clean_up().await;
}
