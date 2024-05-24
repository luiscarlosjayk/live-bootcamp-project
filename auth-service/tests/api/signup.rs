use auth_service::{routes::SignupResponse, ErrorResponse};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let body = serde_json::json!({
        "email": random_email,
        "password": "abcdefgH123",
        "requires2FA": true,
    });
    let response = app.signup(&body).await;

    assert_eq!(response.status().as_u16(), 201);

    let expected_response = SignupResponse {
        message: "User created successfully!".to_owned(),
    };

    // Assert that we are getting the correct response body!
    assert_eq!(
        response
            .json::<SignupResponse>()
            .await
            .expect("Could not deserialize response body to UserBody"),
        expected_response
    );
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    // The signup route should return a 400 HTTP status code if an invalid input is sent.
    // The input is considered invalid if:
    // - The email is empty or does not contain '@'
    // - The password is less than 8 characters

    // Create an array of invalid inputs. Then, iterate through the array and
    // make HTTP calls to the signup route. Assert a 400 HTTP status code is returned.
    let app = TestApp::new().await;
    let invalid_inputs = [
        serde_json::json!({
            "email": "email1", // Doesn't contain @
            "password": "12345678",
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": "", // Empty email
            "password": "12345678",
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": "email3@test.com",
            "password": "1234567", // Password has less than 8 characters
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": "email3@test.com",
            "password": "1234567", // Password has less than 8 characters
            "requires2FA": true,
        }),
    ];

    for invalid_body in invalid_inputs.iter() {
        let response = app.signup(invalid_body).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            invalid_body
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_owned()
        );
    }
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    // Call the signup route twice. The second request should fail with a 409 HTTP status code
    let app = TestApp::new().await;
    let body = serde_json::json!({
        "email": "testuser409@test.com",
        "password": "123456789",
        "requires2FA": true,
    });

    let response = app.signup(&body).await;
    assert_eq!(response.status().as_u16(), 201);
    let response = app.signup(&body).await;
    assert_eq!(response.status().as_u16(), 409);
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );
}
