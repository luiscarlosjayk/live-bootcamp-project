use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_200_if_deletes_user_successfully() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "abcDEF123",
        "requires2FA": true,
        "recaptcha": "recaptcha",
    });

    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let delete_response = app.delete_user(random_email).await;
    assert_eq!(delete_response.status().as_u16(), 200);

    // Clean up database
    app.clean_up().await;
    
}
