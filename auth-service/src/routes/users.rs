use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

pub async fn delete(
    Path(request_email): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(request_email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let mut user_store = state.user_store.write().await;

    if let Ok(user) = user_store.get_user(&email).await {
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
