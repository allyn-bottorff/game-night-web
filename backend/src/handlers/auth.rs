// handlers/auth.rs
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bcrypt::{hash, verify};
use serde_json::json;

use crate::{
    auth::generate_token,
    db::{create_user, get_user_by_username},
    models::{AuthResponse, ErrorResponse, LoginRequest, RegisterRequest, UserInfo},
    AppState,
};

// Login handler
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Response {
    // Check if the user exists
    let user = match get_user_by_username(&state.db, &payload.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid username or password".to_string(),
                }),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Verify the password
    let is_valid = match verify(&payload.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Password verification failed".to_string(),
                }),
            )
                .into_response();
        }
    };

    if !is_valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid username or password".to_string(),
            }),
        )
            .into_response();
    }

    // Generate a JWT token
    let token = match generate_token(&user) {
        Ok(token) => token,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Token generation failed".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Return the token and user info
    (
        StatusCode::OK,
        Json(AuthResponse {
            token,
            user: UserInfo::from(user),
        }),
    )
        .into_response()
}

// Register handler
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Response {
    // Check if the username already exists
    match get_user_by_username(&state.db, &payload.username).await {
        Ok(Some(_)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Username already taken".to_string(),
                }),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                }),
            )
                .into_response();
        }
        Ok(None) => {}
    }

    // Hash the password
    let password_hash = match hash(&payload.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Password hashing failed".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Create the user (non-admin by default)
    let user = match create_user(&state.db, &payload.username, &password_hash, false).await {
        Ok(user) => user,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "User creation failed".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Generate a JWT token
    let token = match generate_token(&user) {
        Ok(token) => token,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Token generation failed".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Return the token and user info
    (
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: UserInfo::from(user),
        }),
    )
        .into_response()
}