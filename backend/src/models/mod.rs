// models/mod.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// User model
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String, // Stores the bcrypt hash of the password
    pub created_at: DateTime<Utc>,
    pub is_admin: bool,
}

// Public user info (without sensitive data)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub is_admin: bool,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            created_at: user.created_at,
            is_admin: user.is_admin,
        }
    }
}

// Poll model
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Poll {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub created_by: i64, // User ID of the creator
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

// Poll option model
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct PollOption {
    pub id: i64,
    pub poll_id: i64,
    pub text: String,
    pub datetime_option: Option<DateTime<Utc>>, // For calendar date/time options
}

// Vote model
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Vote {
    pub id: i64,
    pub poll_id: i64,
    pub option_id: i64,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
}

// Poll result model
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PollResult {
    pub poll: Poll,
    pub options: Vec<PollOptionWithVotes>,
    pub total_votes: i64,
    pub user_vote: Option<i64>, // Option ID the requesting user voted for, if any
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PollOptionWithVotes {
    pub id: i64,
    pub text: String,
    pub datetime_option: Option<DateTime<Utc>>,
    pub vote_count: i64,
    pub percentage: f64,
}

// Request DTOs
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePollRequest {
    pub title: String,
    pub description: Option<String>,
    pub options: Vec<CreatePollOptionRequest>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePollOptionRequest {
    pub text: String,
    pub datetime_option: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub option_id: i64,
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct PollResponse {
    pub poll: Poll,
    pub options: Vec<PollOption>,
    pub user_vote: Option<i64>, // Option ID the requesting user voted for, if any
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}