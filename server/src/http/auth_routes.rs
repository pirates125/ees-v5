use crate::auth::{create_token, hash_password, verify_password};
use crate::db::users;
use crate::http::{ApiError, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // User'ı bul
    let user = users::get_user_by_email(&state.db_pool, &req.email)
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?
        .ok_or_else(|| ApiError::LoginFailed("Kullanıcı bulunamadı".to_string()))?;
    
    // Password kontrol
    if !verify_password(&req.password, &user.password_hash)
        .map_err(|e| ApiError::LoginFailed(e.to_string()))?
    {
        return Err(ApiError::LoginFailed("Şifre hatalı".to_string()));
    }
    
    // Last login güncelle
    users::update_last_login(&state.db_pool, &user.id)
        .await
        .ok();
    
    // JWT token oluştur
    let token = create_token(&user.id, &user.email, &user.role, &state.jwt_secret)
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    let response = LoginResponse {
        token,
        user: UserInfo {
            id: user.id.clone(),
            email: user.email,
            name: user.name,
            role: user.role,
        },
    };
    
    Ok((StatusCode::OK, Json(response)))
}

pub async fn register_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Password hash
    let password_hash = hash_password(&req.password)
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    // User oluştur
    let user = users::create_user(
        &state.db_pool,
        &req.email,
        &password_hash,
        &req.name,
        "agent", // Default role
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate") {
            ApiError::FormValidation("Bu e-posta zaten kayıtlı".to_string())
        } else {
            ApiError::Unknown(e.to_string())
        }
    })?;
    
    // Token oluştur
    let token = create_token(&user.id, &user.email, &user.role, &state.jwt_secret)
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    let response = LoginResponse {
        token,
        user: UserInfo {
            id: user.id.clone(),
            email: user.email,
            name: user.name,
            role: user.role,
        },
    };
    
    Ok((StatusCode::CREATED, Json(response)))
}

