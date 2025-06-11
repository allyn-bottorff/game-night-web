//! # Database Module
//!
//! This module handles all database operations for the Game Night application,
//! including connection pooling, initialization, migrations, and Prometheus metrics.
//!
//! ## Key Components
//! - SQLite connection pool management
//! - Default admin user initialization
//! - Database connection request guard
//! - Prometheus metrics collection and reporting
//!
//! ## Database Configuration
//! The database connection uses the `DATABASE_URL` environment variable,
//! defaulting to `sqlite:game_night.db` if not specified.
//!
//! ## Metrics
//! This module exposes Prometheus metrics for monitoring:
//! - Active and total poll counts
//! - Total votes and users
//! - Login attempt statistics

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::env;
use std::time::Duration;
use crate::models::User;
use lazy_static::lazy_static;
use prometheus::{
    Encoder, IntCounter, IntGauge, TextEncoder, register_int_counter, register_int_gauge,
};

/// Wrapper around a SQLite database connection for use as a Rocket request guard.
/// 
/// This struct implements Rocket's `FromRequest` trait to provide automatic
/// database connection injection into route handlers.
pub struct DbConn(pub sqlx::pool::PoolConnection<sqlx::Sqlite>);

/// Initializes and returns a SQLite connection pool.
/// 
/// This function creates a connection pool with the following configuration:
/// - Maximum 5 concurrent connections
/// - 3-second connection acquisition timeout
/// - Automatic database file creation if missing
/// 
/// # Environment Variables
/// - `DATABASE_URL` - Database connection string (defaults to "sqlite:game_night.db")
/// 
/// # Returns
/// A configured SQLite connection pool ready for use
/// 
/// # Panics
/// Panics if unable to establish database connection
pub async fn init_pool() -> SqlitePool {
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:game_night.db".to_string());

    // Extract the database filename from the URL
    let db_filename = if database_url.starts_with("sqlite:") {
        &database_url[7..]
    } else {
        "game_night.db"
    };

    log::info!("Connecting to database at: {}", db_filename);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(db_filename)
                .create_if_missing(true)
        )
        .await;

    match pool {
        Ok(pool) => {
            log::info!("Successfully connected to SQLite database");
            pool
        }
        Err(err) => {
            log::error!("Failed to connect to SQLite database: {}", err);
            panic!("Failed to connect to SQLite database: {}", err);
        }
    }
}

/// Initializes a default admin user if no admin users exist in the database.
/// 
/// This function ensures there's always at least one admin user in the system
/// for initial setup and management. The default credentials are:
/// - Username: "admin"
/// - Password: "admin"
/// 
/// # Security Note
/// The default password should be changed immediately after first login.
/// A warning is logged to remind administrators of this requirement.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// 
/// # Returns
/// * `Ok(())` - Admin initialization completed successfully
/// * `Err(sqlx::Error)` - Database error during initialization
pub async fn init_default_admin(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Check if any admin users exist
    let admin_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_admin = 1")
        .fetch_one(pool)
        .await?;

    if admin_count == 0 {
        log::info!("No admin users found. Creating default admin user...");
        
        // Create default admin user with password 'admin'
        let default_password = "admin";
        let password_hash = match User::hash_password(default_password) {
            Ok(hash) => hash,
            Err(err) => {
                log::error!("Failed to hash default admin password: {}", err);
                return Err(sqlx::Error::Protocol("Failed to hash password".into()));
            }
        };

        sqlx::query(
            "INSERT INTO users (username, password_hash, is_admin) VALUES ('admin', ?, 1)"
        )
        .bind(&password_hash)
        .execute(pool)
        .await?;

        log::info!("✅ Default admin user created successfully (username: 'admin', password: 'admin')");
        log::warn!("⚠️  Please change the default admin password after first login!");
    } else {
        log::info!("Admin users already exist. Skipping default admin creation.");
    }

    Ok(())
}

// /// Database initialization hook for Rocket
// pub fn init_db() -> AdHoc {
//     AdHoc::try_on_ignite("SQLite Database", |rocket| async {
//         let pool = init_pool().await;

//         // Run migrations
//         sqlx::migrate!("./migrations")
//             .run(&pool)
//             .await
//             .expect("Failed to run database migrations");

//         Ok(rocket.manage(pool))
//     })
// }

/// Retrieve a database connection from the managed pool
// pub async fn get_conn(pool: &SqlitePool) -> Result<DbConn, sqlx::Error> {
//     let conn = pool.acquire().await?;
//     Ok(DbConn(conn))
// }

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for DbConn {
    type Error = ();

    /// Extracts a database connection from the managed pool for use in route handlers.
    /// 
    /// This implementation allows route handlers to receive a `DbConn` parameter
    /// and automatically get a database connection from the managed pool.
    /// 
    /// # Returns
    /// * `Outcome::Success(DbConn)` - Successfully acquired database connection
    /// * `Outcome::Error(ServiceUnavailable)` - Failed to acquire connection
    async fn from_request(
        request: &'r rocket::request::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let pool = request.rocket().state::<SqlitePool>().unwrap();
        match pool.acquire().await {
            Ok(conn) => rocket::request::Outcome::Success(DbConn(conn)),
            Err(_) => {
                rocket::request::Outcome::Error((rocket::http::Status::ServiceUnavailable, ()))
            }
        }
    }
}

// ============================================================================
// Prometheus Metrics
// ============================================================================

/// Global Prometheus metrics for monitoring application performance and usage.
/// These metrics are automatically updated and exposed at the `/metrics` endpoint.
lazy_static! {
    static ref ACTIVE_POLLS: IntGauge =
        register_int_gauge!("game_night_active_polls", "Number of active polls").unwrap();
    static ref TOTAL_POLLS: IntGauge =
        register_int_gauge!("game_night_total_polls", "Total number of polls").unwrap();
    static ref TOTAL_VOTES: IntGauge =
        register_int_gauge!("game_night_total_votes", "Total number of votes cast").unwrap();
    static ref TOTAL_USERS: IntGauge =
        register_int_gauge!("game_night_total_users", "Total number of registered users").unwrap();
    static ref LOGIN_ATTEMPTS: IntCounter =
        register_int_counter!("game_night_login_attempts", "Number of login attempts").unwrap();
    static ref SUCCESSFUL_LOGINS: IntCounter = register_int_counter!(
        "game_night_successful_logins",
        "Number of successful logins"
    )
    .unwrap();
    static ref FAILED_LOGINS: IntCounter =
        register_int_counter!("game_night_failed_logins", "Number of failed logins").unwrap();
    static ref API_REQUESTS: IntCounter =
        register_int_counter!("game_night_api_requests", "Number of API requests").unwrap();
}

/// Updates all database-derived metrics by querying current counts.
/// 
/// This function refreshes the Prometheus metrics with current database
/// statistics including poll counts, vote counts, and user counts.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// 
/// # Returns
/// * `Ok(())` - Metrics updated successfully
/// * `Err(sqlx::Error)` - Database error during metric collection
pub async fn update_metrics(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Get active polls count
    let active_polls: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM polls WHERE expires_at > datetime('now')")
            .fetch_one(pool)
            .await?;
    ACTIVE_POLLS.set(active_polls);

    // Get total polls count
    let total_polls: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM polls")
        .fetch_one(pool)
        .await?;
    TOTAL_POLLS.set(total_polls);

    // Get total votes count
    let total_votes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM votes")
        .fetch_one(pool)
        .await?;
    TOTAL_VOTES.set(total_votes);

    // Get total users count
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    TOTAL_USERS.set(total_users);

    Ok(())
}

/// Increments the total login attempts counter.
/// 
/// This function should be called every time a user attempts to log in,
/// regardless of whether the attempt succeeds or fails.
pub fn increment_login_attempt() {
    LOGIN_ATTEMPTS.inc();
}

/// Increments the successful logins counter.
/// 
/// This function should be called when a user successfully authenticates
/// and a session is established.
pub fn increment_successful_login() {
    SUCCESSFUL_LOGINS.inc();
}

/// Increments the failed logins counter.
/// 
/// This function should be called when a login attempt fails due to
/// invalid credentials or other authentication errors.
pub fn increment_failed_login() {
    FAILED_LOGINS.inc();
}

// Increment the API requests counter
// pub fn increment_api_request() {
//     API_REQUESTS.inc();
// }

/// Generates a Prometheus-formatted metrics response.
/// 
/// This function updates all metrics from the database and returns
/// a text response in Prometheus exposition format suitable for
/// scraping by monitoring systems.
/// 
/// # Arguments
/// * `pool` - Database connection pool for updating metrics
/// 
/// # Returns
/// String containing Prometheus-formatted metrics data
pub async fn get_metrics(pool: &SqlitePool) -> String {
    // Update metrics from database
    let _ = update_metrics(pool).await;

    // Gather all registered metrics
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    String::from_utf8(buffer).unwrap()
}

// Middleware for tracking API requests
// pub fn track_request(_request: &Request) {
//     increment_api_request();
// }
