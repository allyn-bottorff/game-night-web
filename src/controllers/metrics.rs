use lazy_static::lazy_static;
use prometheus::{
    Encoder, IntCounter, IntGauge, TextEncoder, register_int_counter, register_int_gauge,
};
// use rocket::request::Request;
use sqlx::SqlitePool;

// Define the metrics we want to track
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

// Function to gather metrics from the database
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

// Increment the login attempts counter
pub fn increment_login_attempt() {
    LOGIN_ATTEMPTS.inc();
}

// Increment the successful logins counter
pub fn increment_successful_login() {
    SUCCESSFUL_LOGINS.inc();
}

// Increment the failed logins counter
pub fn increment_failed_login() {
    FAILED_LOGINS.inc();
}

// Increment the API requests counter
// pub fn increment_api_request() {
//     API_REQUESTS.inc();
// }

// Generate the metrics response
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
