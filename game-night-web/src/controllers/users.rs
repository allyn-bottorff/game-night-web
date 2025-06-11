//! # User Controller Module
//!
//! This module contains all business logic related to user management,
//! including authentication, user creation, password changes, and role management.
//!
//! ## Key Functions
//! - User login and logout
//! - User account creation (admin only)
//! - Password change functionality
//! - User role management (admin promotion/demotion)
//! - User statistics and profile information

use rocket::http::CookieJar;
use rocket::response::{Flash, Redirect};
use rocket::uri;
use sqlx::SqlitePool;
use log::{info, error};

use crate::models::{User, LoginForm, NewUserForm, ChangePasswordForm};
use crate::auth::{login_user, set_login_cookie, clear_login_cookie};

/// Handles user login authentication and session creation.
/// 
/// This function verifies the user's credentials against the database,
/// sets a session cookie upon successful authentication, and redirects
/// to the dashboard.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `form` - Login form data containing username and password
/// * `cookies` - Cookie jar for setting session cookies
/// 
/// # Returns
/// * `Ok(Redirect)` - Redirects to dashboard on successful login
/// * `Err(Flash<Redirect>)` - Redirects to login page with error message
pub async fn login_controller(
    pool: &SqlitePool,
    form: &LoginForm,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Flash<Redirect>> {
    match login_user(pool, &form.username, &form.password).await {
        Ok(user) => {
            info!("User logged in: {}", user.username);
            set_login_cookie(cookies, user.id);
            Ok(Redirect::to(uri!(crate::routes::dashboard)))
        }
        Err(err) => {
            error!("Login error: {}", err);
            Err(Flash::error(
                Redirect::to(uri!(crate::routes::login_page)),
                format!("Login failed: {}", err),
            ))
        }
    }
}

/// Handles user logout by clearing the session cookie.
/// 
/// This function removes the user's session cookie and redirects
/// to the login page with a success message.
/// 
/// # Arguments
/// * `cookies` - Cookie jar for clearing session cookies
/// 
/// # Returns
/// A flash redirect to the login page with logout confirmation
pub fn logout_controller(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    clear_login_cookie(cookies);
    Flash::success(
        Redirect::to(uri!(crate::routes::login_page)),
        "You have been logged out.",
    )
}

/// Creates a new user account (admin functionality).
/// 
/// This function validates the form data, checks for existing users,
/// hashes the password, and creates a new user account in the database.
/// 
/// # Validation Steps
/// 1. Checks for empty username or password
/// 2. Verifies password confirmation matches
/// 3. Ensures username doesn't already exist
/// 4. Hashes the password securely
/// 5. Inserts the new user into the database
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `form` - New user form data
/// 
/// # Returns
/// * `Ok(Flash<Redirect>)` - Success redirect to admin users page
/// * `Err(Flash<Redirect>)` - Error redirect to add user page with message
pub async fn add_user_controller(
    pool: &SqlitePool,
    form: &NewUserForm,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    // Validate form inputs
    if form.username.trim().is_empty() {
        return Err(Flash::error(
            Redirect::to(uri!(crate::routes::add_user_page)),
            "Username cannot be empty.",
        ));
    }

    if form.password.trim().is_empty() {
        return Err(Flash::error(
            Redirect::to(uri!(crate::routes::add_user_page)),
            "Password cannot be empty.",
        ));
    }

    if form.password != form.confirm_password {
        return Err(Flash::error(
            Redirect::to(uri!(crate::routes::add_user_page)),
            "Passwords do not match.",
        ));
    }

    // Check if user already exists
    let existing_user = sqlx::query("SELECT id FROM users WHERE username = ?")
        .bind(&form.username)
        .fetch_optional(pool)
        .await;

    match existing_user {
        Ok(Some(_)) => {
            return Err(Flash::error(
                Redirect::to(uri!(crate::routes::add_user_page)),
                "Username already exists.",
            ));
        }
        Err(err) => {
            error!("Database error checking user: {}", err);
            return Err(Flash::error(
                Redirect::to(uri!(crate::routes::add_user_page)),
                "Database error occurred.",
            ));
        }
        _ => {}
    }

    // Hash the password
    let password_hash = match User::hash_password(&form.password) {
        Ok(hash) => hash,
        Err(err) => {
            error!("Error hashing password: {}", err);
            return Err(Flash::error(
                Redirect::to(uri!(crate::routes::add_user_page)),
                "Error creating user account.",
            ));
        }
    };

    // Insert the new user
    let result = sqlx::query(
        "INSERT INTO users (username, password_hash, is_admin) VALUES (?, ?, ?)",
    )
    .bind(&form.username)
    .bind(&password_hash)
    .bind(form.is_admin)
    .execute(pool)
    .await;

    match result {
        Ok(_) => {
            info!("New user created: {}", form.username);
            Ok(Flash::success(
                Redirect::to(uri!(crate::routes::admin_users)),
                format!("User {} created successfully.", form.username),
            ))
        }
        Err(err) => {
            error!("Error creating user: {}", err);
            Err(Flash::error(
                Redirect::to(uri!(crate::routes::add_user_page)),
                "Error creating user account.",
            ))
        }
    }
}

/// Retrieves user statistics for profile display.
/// 
/// This function queries the database to get the number of polls
/// created by the user and the number of votes they have cast.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - ID of the user to get statistics for
/// 
/// # Returns
/// * `Ok((polls_created, votes_cast))` - Tuple of user statistics
/// * `Err(sqlx::Error)` - Database error if query fails
pub async fn get_user_stats(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<(i64, i64), sqlx::Error> {
    // Get count of polls created by user
    let polls_created = sqlx::query_scalar("SELECT COUNT(*) FROM polls WHERE creator_id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    
    // Get count of votes cast by user
    let votes_cast = sqlx::query_scalar("SELECT COUNT(*) FROM votes WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    
    Ok((polls_created, votes_cast))
}

/// Handles user password change requests.
/// 
/// This function validates the current password, checks the new password
/// confirmation, hashes the new password, and updates it in the database.
/// 
/// # Validation Steps
/// 1. Ensures new password is not empty
/// 2. Verifies new password confirmation matches
/// 3. Retrieves current user data from database
/// 4. Verifies current password is correct
/// 5. Hashes the new password
/// 6. Updates the password in the database
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - ID of the user changing their password
/// * `form` - Password change form data
/// 
/// # Returns
/// * `Ok(Flash<Redirect>)` - Success redirect to profile page
/// * `Err(Flash<Redirect>)` - Error redirect to profile page with message
pub async fn change_password(
    pool: &SqlitePool,
    user_id: i64,
    form: &ChangePasswordForm,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    // Verify form data
    if form.new_password.trim().is_empty() {
        return Err(Flash::error(
            Redirect::to(uri!(crate::routes::profile)),
            "New password cannot be empty.",
        ));
    }
    
    if form.new_password != form.confirm_password {
        return Err(Flash::error(
            Redirect::to(uri!(crate::routes::profile)),
            "New passwords do not match.",
        ));
    }
    
    // Get current user data
    let user = match sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, is_admin, created_at FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await {
        Ok(user) => user,
        Err(err) => {
            error!("Database error fetching user: {}", err);
            return Err(Flash::error(
                Redirect::to(uri!(crate::routes::profile)),
                "Error retrieving user account.",
            ));
        }
    };
    
    // Verify current password
    if !user.verify_password(&form.current_password) {
        return Err(Flash::error(
            Redirect::to(uri!(crate::routes::profile)),
            "Current password is incorrect.",
        ));
    }
    
    // Hash the new password
    let password_hash = match User::hash_password(&form.new_password) {
        Ok(hash) => hash,
        Err(err) => {
            error!("Error hashing password: {}", err);
            return Err(Flash::error(
                Redirect::to(uri!(crate::routes::profile)),
                "Error updating password.",
            ));
        }
    };
    
    // Update the password
    let result = sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&password_hash)
        .bind(user_id)
        .execute(pool)
        .await;
    
    match result {
        Ok(_) => {
            info!("Password updated for user ID: {}", user_id);
            Ok(Flash::success(
                Redirect::to(uri!(crate::routes::profile)),
                "Your password has been updated successfully.",
            ))
        }
        Err(err) => {
            error!("Error updating password: {}", err);
            Err(Flash::error(
                Redirect::to(uri!(crate::routes::profile)),
                "Error updating password.",
            ))
        }
    }
}

/// Retrieves a list of all users in the system (admin functionality).
/// 
/// This function queries the database for all users and returns them
/// ordered by username for display in the admin users page.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// 
/// # Returns
/// * `Ok(Vec<User>)` - Vector of all users in the system
/// * `Err(sqlx::Error)` - Database error if query fails
pub async fn get_all_users(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, is_admin, created_at FROM users ORDER BY username",
    )
    .fetch_all(pool)
    .await
}

/// Toggles admin role for a user (admin functionality).
/// 
/// This function allows administrators to promote users to admin status
/// or demote them to regular user status. It includes safety checks to
/// prevent admins from changing their own role.
/// 
/// # Safety Checks
/// 1. Prevents users from changing their own role
/// 2. Verifies the target user exists
/// 3. Updates the user's admin status in the database
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - ID of the user whose role should be changed
/// * `set_admin` - Whether to set admin privileges (true) or remove them (false)
/// * `admin_id` - ID of the admin performing the action
/// 
/// # Returns
/// * `Ok(Flash<Redirect>)` - Success redirect to admin users page
/// * `Err(Flash<Redirect>)` - Error redirect to admin users page with message
pub async fn toggle_user_role(
    pool: &SqlitePool,
    user_id: i64,
    set_admin: bool,
    admin_id: i64,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    // Don't allow users to change their own role
    if user_id == admin_id {
        return Err(Flash::error(
            Redirect::to(uri!(crate::routes::admin_users)),
            "You cannot change your own role.",
        ));
    }
    
    // Check if user exists
    let user_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await;
    
    match user_exists {
        Ok(true) => {
            // Update user role
            let result = sqlx::query(
                "UPDATE users SET is_admin = ? WHERE id = ?",
            )
            .bind(set_admin)
            .bind(user_id)
            .execute(pool)
            .await;
            
            match result {
                Ok(_) => {
                    let role_str = if set_admin { "admin" } else { "user" };
                    info!("User role updated: user_id={}, new_role={}", user_id, role_str);
                    Ok(Flash::success(
                        Redirect::to(uri!(crate::routes::admin_users)),
                        format!("User role updated to {}.", role_str),
                    ))
                }
                Err(err) => {
                    error!("Database error updating role: {}", err);
                    Err(Flash::error(
                        Redirect::to(uri!(crate::routes::admin_users)),
                        "Error updating user role.",
                    ))
                }
            }
        }
        Ok(false) => {
            error!("Attempted to change role for non-existent user: {}", user_id);
            Err(Flash::error(
                Redirect::to(uri!(crate::routes::admin_users)),
                "User not found.",
            ))
        }
        Err(err) => {
            error!("Database error checking user: {}", err);
            Err(Flash::error(
                Redirect::to(uri!(crate::routes::admin_users)),
                "Database error occurred.",
            ))
        }
    }
}