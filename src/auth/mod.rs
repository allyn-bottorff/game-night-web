use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::{Flash, Redirect};
use rocket::serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::ops::Deref;

use crate::models::user::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user: User,
}

impl Deref for AuthenticatedUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub user: User,
}

impl Deref for AdminUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Get the user_id from the cookies
        let cookies = request.cookies();
        let user_id = cookies
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse::<i64>().ok());

        if let Some(user_id) = user_id {
            // Get the database connection
            let pool = request.rocket().state::<SqlitePool>().unwrap();

            // Fetch the user from the database
            let user_result = sqlx::query_as::<_, User>(
                "SELECT id, username, password_hash, is_admin, created_at FROM users WHERE id = ?",
            )
            .bind(user_id)
            .fetch_one(pool)
            .await;

            match user_result {
                Ok(user) => Outcome::Success(AuthenticatedUser { user }),
                Err(_) => {
                    cookies.remove_private(Cookie::named("user_id"));
                    Outcome::Failure((Status::Unauthorized, ()))
                }
            }
        } else {
            Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_outcome = request.guard::<AuthenticatedUser>().await;

        match user_outcome {
            Outcome::Success(auth_user) if auth_user.is_admin => {
                Outcome::Success(AdminUser { user: auth_user.user })
            }
            _ => Outcome::Failure((Status::Forbidden, ())),
        }
    }
}

// Authentication functions
pub fn set_login_cookie(cookies: &CookieJar<'_>, user_id: i64) {
    cookies.add_private(Cookie::new("user_id", user_id.to_string()));
}

pub fn clear_login_cookie(cookies: &CookieJar<'_>) {
    cookies.remove_private(Cookie::named("user_id"));
}

// Login handler with database verification
pub async fn login_user(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<User, &'static str> {
    let user_result = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, is_admin, created_at FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await;

    match user_result {
        Ok(Some(user)) if user.verify_password(password) => Ok(user),
        Ok(Some(_)) => Err("Invalid password"),
        Ok(None) => Err("User not found"),
        Err(_) => Err("Database error"),
    }
}