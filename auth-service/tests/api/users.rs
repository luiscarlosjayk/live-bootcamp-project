use std::ops::Range;

use fake::Fake;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_200_if_deletes_user_successfully() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let random_password =
        fake::faker::internet::en::Password(Range { start: 8, end: 12 }).fake::<String>();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": random_password,
        "requires2FA": true,
        "recaptcha": "recaptcha",
    });
    let signup_response = app.signup(&signup_body).await;

    assert_eq!(signup_response.status().as_u16(), 201);

    let delete_response = app.delete_user(random_email).await;

    assert_eq!(delete_response.status().as_u16(), 200);
}
