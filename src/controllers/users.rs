use rocket::http::CookieJar;
use rocket::response::{Flash, Redirect};
use rocket::uri;
use sqlx::SqlitePool;
use log::{info, error};

use crate::models::user::{User, LoginForm, NewUserForm};
use crate::auth::{login_user, set_login_cookie, clear_login_cookie};

// Attempt to login a user
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

// Log out a user
pub fn logout_controller(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    clear_login_cookie(cookies);
    Flash::success(
        Redirect::to(uri!(crate::routes::login_page)),
        "You have been logged out.",
    )
}

// Create a new user
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

// Get list of all users
pub async fn get_all_users(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, is_admin, created_at FROM users ORDER BY username",
    )
    .fetch_all(pool)
    .await
}