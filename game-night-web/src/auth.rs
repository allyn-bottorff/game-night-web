//! # Authentication and Authorization Module
//!
//! This module provides authentication and authorization functionality for the Game Night application.
//! It includes session-based authentication using Rocket's private cookies, role-based access control,
//! and user authentication functions.
//!
//! ## Key Components
//! - [`AuthenticatedUser`] - Request guard for authenticated users
//! - [`AdminUser`] - Request guard for admin users only
//! - Cookie-based session management functions
//! - User login verification
//!
//! ## Authentication Flow
//! 1. User submits credentials via login form
//! 2. Credentials are verified against database
//! 3. Upon success, encrypted session cookie is set
//! 4. Subsequent requests use cookie for authentication
//! 5. Request guards automatically validate sessions

use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::ops::Deref;

use crate::models::User;

/// Request guard that represents an authenticated user.
/// 
/// This struct wraps a User and is used as a request guard to ensure
/// that only authenticated users can access certain endpoints.
/// 
/// # Usage
/// ```rust
/// #[get("/protected")]
/// fn protected_route(user: AuthenticatedUser) -> String {
///     format!("Hello, {}!", user.username)
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    /// The authenticated user's information
    pub user: User,
}

impl Deref for AuthenticatedUser {
    type Target = User;

    /// Allows direct access to User fields through the AuthenticatedUser.
    /// This enables using `auth_user.username` instead of `auth_user.user.username`.
    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

/// Request guard that represents an authenticated admin user.
/// 
/// This struct wraps a User and is used as a request guard to ensure
/// that only authenticated admin users can access certain endpoints.
/// 
/// # Usage
/// ```rust
/// #[get("/admin")]
/// fn admin_route(admin: AdminUser) -> String {
///     format!("Admin panel for {}", admin.username)
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    /// The authenticated admin user's information
    pub user: User,
}

impl Deref for AdminUser {
    type Target = User;

    /// Allows direct access to User fields through the AdminUser.
    /// This enables using `admin_user.username` instead of `admin_user.user.username`.
    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    /// Extracts an authenticated user from the request.
    /// 
    /// This method checks for a valid session cookie, retrieves the user from
    /// the database, and returns an AuthenticatedUser if successful.
    /// 
    /// # Authentication Process
    /// 1. Extract user_id from encrypted session cookie
    /// 2. Query database for user with that ID
    /// 3. Return Success if user found, Error otherwise
    /// 4. Clean up invalid cookies if user lookup fails
    /// 
    /// # Returns
    /// - `Outcome::Success(AuthenticatedUser)` if authentication succeeds
    /// - `Outcome::Error(Unauthorized)` if authentication fails
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
                    cookies.remove_private(Cookie::from("user_id"));
                    Outcome::Error((Status::Unauthorized, ()))
                }
            }
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = ();

    /// Extracts an authenticated admin user from the request.
    /// 
    /// This method first checks if the user is authenticated, then verifies
    /// they have admin privileges before allowing access.
    /// 
    /// # Authentication Process
    /// 1. Use AuthenticatedUser guard to verify authentication
    /// 2. Check if the authenticated user has admin privileges
    /// 3. Return Success if user is admin, Forbidden otherwise
    /// 
    /// # Returns
    /// - `Outcome::Success(AdminUser)` if user is authenticated admin
    /// - `Outcome::Error(Forbidden)` if user lacks admin privileges
    /// - Inherits authentication errors from AuthenticatedUser guard
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_outcome = request.guard::<AuthenticatedUser>().await;

        match user_outcome {
            Outcome::Success(auth_user) if auth_user.is_admin => Outcome::Success(AdminUser {
                user: auth_user.user,
            }),
            _ => Outcome::Error((Status::Forbidden, ())),
        }
    }
}

// ============================================================================
// Authentication utility functions
// ============================================================================
/// Sets an encrypted session cookie for the authenticated user.
/// 
/// This function creates a private (encrypted) cookie containing the user's ID
/// that will be used for subsequent authentication checks.
/// 
/// # Arguments
/// * `cookies` - The cookie jar from the current request
/// * `user_id` - The ID of the user to authenticate
/// 
/// # Security Note
/// The cookie is encrypted using Rocket's private cookie functionality,
/// which requires a valid ROCKET_SECRET_KEY in the environment.
pub fn set_login_cookie(cookies: &CookieJar<'_>, user_id: i64) {
    cookies.add_private(Cookie::new("user_id", user_id.to_string()));
}

/// Removes the session cookie, effectively logging out the user.
/// 
/// This function removes the encrypted session cookie, which will cause
/// subsequent requests to be treated as unauthenticated.
/// 
/// # Arguments
/// * `cookies` - The cookie jar from the current request
pub fn clear_login_cookie(cookies: &CookieJar<'_>) {
    cookies.remove_private(Cookie::from("user_id"));
}

/// Verifies user credentials and returns the authenticated user.
/// 
/// This function performs the core authentication logic by looking up
/// the user in the database and verifying their password hash.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `username` - Username to authenticate
/// * `password` - Plain text password to verify
/// 
/// # Returns
/// * `Ok(User)` - If authentication succeeds
/// * `Err(&'static str)` - If authentication fails with error message
/// 
/// # Errors
/// * "User not found" - Username doesn't exist in database
/// * "Invalid password" - Password doesn't match stored hash
/// * "Database error" - Database query failed
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