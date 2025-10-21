use crate::auth::jwt::{verify_token, Claims};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(
    State(jwt_secret): State<String>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Authorization header'dan token al
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Token'ı doğrula
    let claims = verify_token(token, &jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Claims'i request'e ekle
    req.extensions_mut().insert(claims);
    
    Ok(next.run(req).await)
}

// Admin middleware
pub async fn admin_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    if claims.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(req).await)
}

