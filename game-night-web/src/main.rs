//! # Game Night Web Application
//!
//! A web application for managing game night polls and user voting.
//! Built with Rocket framework and SQLite database.
//!
//! ## Features
//! - User authentication with role-based access control
//! - Poll creation and voting system
//! - Admin user management
//! - Session-based authentication
//! - Prometheus metrics collection

extern crate rocket;
use dotenv::dotenv;
use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer};
use rocket::response::Redirect;
use rocket::{catch, catchers, uri};
use rocket_dyn_templates::Template;
use std::env;

mod auth;
mod controllers;
mod db;
mod models;
mod routes;

use routes::*;

/// Error catcher for 401 Unauthorized responses.
/// 
/// This catcher intercepts 401 status responses and redirects unauthenticated
/// users to the login page instead of showing a raw error response.
/// 
/// # Returns
/// Redirect to the login page
#[catch(401)]
fn unauthorized() -> Redirect {
    Redirect::to(uri!(login_page))
}

/// Main application entry point that configures and launches the Rocket web server.
/// 
/// This function:
/// - Loads environment variables from .env file
/// - Initializes logging
/// - Sets up all HTTP routes
/// - Configures static file serving
/// - Attaches template engine
/// - Initializes database connection pool
/// - Runs database migrations
/// - Creates default admin user if needed
/// 
/// # Returns
/// A configured Rocket instance ready for launch
#[rocket::launch]
fn rocket() -> _ {
    // Load environment variables
    dotenv().ok();

    // Configure logging
    env_logger::init();

    rocket::build()
        .mount(
            "/",
            rocket::routes![
                index,
                login_page,
                login_post,
                logout,
                dashboard,
                get_polls,
                poll_detail,
                poll_voters,
                create_poll_page,
                create_poll_post,
                vote_on_poll,
                delete_poll,
                profile,
                change_password,
                admin_users,
                add_user_page,
                add_user_post,
                toggle_user_role,
                metrics_endpoint
            ],
        )
        .mount("/static", FileServer::from(relative!("src/static")))
        .register("/", catchers![unauthorized])
        .attach(Template::fairing())
        .attach(AdHoc::try_on_ignite("Database Setup", |rocket| async {
            let pool = db::init_pool().await;

            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("failed to run database migrations");

            // Initialize default admin user if needed
            if let Err(e) = db::init_default_admin(&pool).await {
                log::error!("Failed to initialize default admin user: {}", e);
                panic!("Failed to initialize default admin user: {}", e);
            }

            Ok(rocket.manage(pool))
        }))
}
