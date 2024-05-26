use serde_json::json;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn login_should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;
    let invalid_inputs = [
        json!({
            "email": get_random_email(),
        }),
        json!({
            "password": "abcEFG123"
        }),
        json!({
            "email": get_random_email(),
            "password": 123, // Shouldn't be a number
        }),
        json!({
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
}

#[tokio::test]
async fn login_should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let invalid_inputs = [
        json!({
            "email": get_random_email(),
            "password": "abcdef123", // Should container uppercase
        }),
        json!({
            "email": get_random_email(),
            "password": "abcdDEFG", // Should contain number
        }),
        json!({
            "email": get_random_email(),
            "password": "ABCDEF123", // Should contain lowercase
        }),
        json!({
            "email": get_random_email(),
            "password": "abcDEF", // Should be longer than 8 characters
        }),
        json!({
            "email": "@test.com", // Missing username
            "password": "abcDEF123",
        }),
        json!({
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
    }
}

#[tokio::test]
async fn login_should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;
    let body = json!({
        "email": "unexisting@test.com",
        "password": "abcDEF123",
    }); // This user doesn't exist

    let login_response = app.post_login(&body).await;

    assert_eq!(login_response.status().as_u16(), 401);
}
