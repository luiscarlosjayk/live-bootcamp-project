use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

pub async fn signup(Json(_request): Json<SignupRequest>) -> impl IntoResponse {
    StatusCode::OK.into_response()
}

#[derive(Serialize, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}
