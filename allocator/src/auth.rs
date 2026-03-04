use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::Response,
};

use crate::controller::error::Error;

pub async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, Error> {
    // Extract auth token from app state via extensions
    let auth_token = request
        .extensions()
        .get::<Option<String>>()
        .cloned()
        .flatten();

    // If no auth token is configured, skip authentication
    if auth_token.is_none() {
        return Ok(next.run(request).await);
    }

    let auth_token = auth_token.unwrap();

    // Check Authorization header
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Error::Auth("Missing Authorization header".to_string()))?;

    // Verify Bearer token format
    if !auth_header.starts_with("Bearer ") {
        return Err(Error::Auth("Invalid Authorization header format".to_string()));
    }

    let token = &auth_header[7..]; // Skip "Bearer "

    // Verify token matches
    if token != auth_token {
        return Err(Error::Auth("Invalid token".to_string()));
    }

    Ok(next.run(request).await)
}
