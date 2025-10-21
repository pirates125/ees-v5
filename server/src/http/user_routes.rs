use crate::auth::Claims;
use crate::db::users;
use crate::http::{ApiError, AppState, UserResponse};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn update_profile_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let updated_user = users::update_user_profile(&state.db_pool, &claims.sub, req.name, req.phone)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok((StatusCode::OK, Json(UserResponse::from(updated_user))))
}

pub async fn change_password_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, ApiError> {
    users::change_password(&state.db_pool, &claims.sub, &req.current_password, &req.new_password)
        .await
        .map_err(|e| ApiError::Unauthorized(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

