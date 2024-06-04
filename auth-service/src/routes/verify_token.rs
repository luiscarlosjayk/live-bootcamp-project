use crate::{app_state::AppState, domain::AuthAPIError, utils::auth::validate_token};
use axum::{extract::State, Json};
use reqwest::StatusCode;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct VerifyTokenRequest {
    pub token: String,
}

pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<StatusCode, AuthAPIError> {
    match validate_token(&request.token, state.banned_token_store.clone()).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(AuthAPIError::InvalidToken),
    }
}
