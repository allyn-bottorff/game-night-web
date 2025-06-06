// handlers/users.rs
use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bcrypt::hash;
use serde::{Deserialize, Serialize};

use crate::{
    AppState,
    db::{create_user, list_users},
    models::{ErrorResponse, UserInfo},
};

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

// List all users
pub async fn list_all_users(
    State(state): State<AppState>,
    Extension(current_user): Extension<UserInfo>,
) -> Response {
    // Only admins can list users
    if !current_user.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Admin privileges required".to_string(),
            }),
        )
            .into_response();
    }

    match list_users(&state.db).await {
        Ok(users) => {
            let user_infos: Vec<UserInfo> = users.into_iter().map(UserInfo::from).collect();
            (StatusCode::OK, Json(user_infos)).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to retrieve users".to_string(),
            }),
        )
            .into_response(),
    }
}

// Create a new user (admin only)
pub async fn create_new_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<UserInfo>,
    Json(payload): Json<CreateUserRequest>,
) -> Response {
    // Only admins can create users
    if !current_user.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Admin privileges required".to_string(),
            }),
        )
            .into_response();
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

    // Create the user
    match create_user(
        &state.db,
        &payload.username,
        &password_hash,
        payload.is_admin,
    )
    .await
    {
        Ok(user) => (StatusCode::CREATED, Json(UserInfo::from(user))).into_response(),
        Err(e) => {
            // Check if it's a unique constraint violation (username already exists)
            if e.to_string().contains("UNIQUE constraint failed") {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Username already taken".to_string(),
                    }),
                )
                    .into_response();
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "User creation failed".to_string(),
                }),
            )
                .into_response()
        }
    }
}
