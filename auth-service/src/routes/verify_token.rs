use axum::{response::IntoResponse, Json};
use reqwest::StatusCode;

use crate::utils::auth::validate_token;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct VerifyTokenRequest {
    pub token: String,
}

pub async fn verify_token(Json(body): Json<VerifyTokenRequest>) -> impl IntoResponse {
    let token = body.token;

    let is_token_valid = validate_token(&token).await;

    if is_token_valid.is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    StatusCode::OK.into_response()
}
