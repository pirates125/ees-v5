use crate::auth::Claims;
use crate::db::quotes;
use crate::http::{ApiError, AppState, UserQuoteResponse};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use uuid::Uuid;

pub async fn list_user_quotes_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, ApiError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| ApiError::Unauthorized(format!("Invalid user ID: {}", e)))?;

    let user_quotes = quotes::list_user_quotes(&state.db_pool, user_id)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let response: Vec<UserQuoteResponse> = user_quotes
        .into_iter()
        .map(|q| UserQuoteResponse::from(q))
        .collect();

    Ok((StatusCode::OK, Json(response)))
}

