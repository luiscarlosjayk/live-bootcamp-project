use crate::{app_state::AppState, utils::auth};
use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct VerifyTokenRequest {
    pub token: String,
}

pub async fn verify_token(
    State(state): State<AppState>,
    Json(body): Json<VerifyTokenRequest>,
) -> impl IntoResponse {
    let token = body.token;
    let banned_token_store = state.banned_token_store;
    let is_token_valid = auth::validate_token(&token, banned_token_store).await;

    if is_token_valid.is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    StatusCode::OK.into_response()
}
