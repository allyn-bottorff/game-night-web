// auth.rs
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

use crate::AppState;
use crate::models::{ErrorResponse, User, UserInfo};

// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID as subject
    pub username: String,
    pub is_admin: bool,
    pub exp: i64, // Expiration time (Unix timestamp)
    pub iat: i64, // Issued at time (Unix timestamp)
}

// Generate a JWT token for a user
pub fn generate_token(user: &User) -> Result<String, jsonwebtoken::errors::Error> {
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "game_night_secret_key".to_string());
    
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();
    
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        is_admin: user.is_admin,
        exp: expiration,
        iat: Utc::now().timestamp(),
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
}

// Authentication middleware
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    // Get the token from the Authorization header
    let auth_header = request.headers().get(header::AUTHORIZATION);
    
    let auth_header = if let Some(header) = auth_header {
        header.to_str().unwrap_or_default()
    } else {
        return unauthorized_response();
    };
    
    // Check if the header starts with "Bearer "
    if !auth_header.starts_with("Bearer ") {
        return unauthorized_response();
    }
    
    // Extract the token
    let token = &auth_header[7..];
    
    // Public endpoints that don't require authentication
    let path = request.uri().path();
    if path == "/api/health" || path == "/api/metrics" || path == "/api/login" || path == "/api/register" {
        return next.run(request).await;
    }
    
    // Validate the token
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "game_night_secret_key".to_string());
    
    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(_) => return unauthorized_response(),
    };
    
    let claims = token_data.claims;
    
    // Get the user from the database to verify they still exist
    let user_id = claims.sub.parse::<i64>().unwrap_or_default();
    let user = match crate::db::get_user_by_id(&state.db, user_id).await {
        Ok(Some(user)) => user,
        _ => return unauthorized_response(),
    };
    
    // Add user info to request extensions
    request.extensions_mut().insert(UserInfo::from(user));
    
    // Continue with the request
    next.run(request).await
}

// Helper function to return unauthorized response
fn unauthorized_response() -> Response {
    let json = Json(ErrorResponse {
        error: "Unauthorized".to_string(),
    });
    (StatusCode::UNAUTHORIZED, json).into_response()
}