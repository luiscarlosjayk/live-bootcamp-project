use crate::{app_state::AppState, domain::AuthAPIError};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use validator::ValidateEmail;

pub async fn delete(
    Path(request_email): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AuthAPIError> {
    if !ValidateEmail::validate_email(&request_email) {
        return Err(AuthAPIError::UserNotFound);
    }

    let mut user_store = state.user_store.write().await;

    if let Ok(user) = user_store.get_user(request_email).await {
        let _ = user_store.delete_user(user).await;
        let response = Json(DeleteUserResponse {
            message: "User deleted successfully!".to_string(),
        });
        Ok((StatusCode::OK, response))
    } else {
        Err(AuthAPIError::UserNotFound)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DeleteUserResponse {
    pub message: String,
}
