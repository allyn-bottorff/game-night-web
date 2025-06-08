use chrono::{DateTime, Utc};
use rocket::form::FromForm;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromForm, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, FromForm, Deserialize)]
pub struct NewUserForm {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
    pub is_admin: bool,
}

#[derive(Debug, FromForm, Deserialize)]
pub struct ChangePasswordForm {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Debug, FromForm, Deserialize)]
pub struct ToggleRoleForm {
    pub user_id: i64,
    pub set_admin: bool,
}

impl User {
    // pub fn new(id: i64, username: String, password_hash: String, is_admin: bool, created_at: DateTime<Utc>) -> Self {
    //     Self {
    //         id,
    //         username,
    //         password_hash,
    //         is_admin,
    //         created_at,
    //     }
    // }

    pub fn verify_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.password_hash).unwrap_or(false)
    }

    pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
        bcrypt::hash(password, 12)
    }
}
