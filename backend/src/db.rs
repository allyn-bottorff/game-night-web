// db.rs
use chrono::{DateTime, Utc};
use sqlx::{Pool, Sqlite};
use tracing::info;

use crate::models::{Poll, PollOption, User, Vote};

// Initialize the database schema
pub async fn init_db(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    info!("Initializing database schema");

    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            is_admin BOOLEAN NOT NULL DEFAULT FALSE
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create polls table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS polls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT,
            created_by INTEGER NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            expires_at TIMESTAMP,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            FOREIGN KEY (created_by) REFERENCES users (id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create poll options table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS poll_options (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            poll_id INTEGER NOT NULL,
            text TEXT NOT NULL,
            datetime_option TIMESTAMP,
            FOREIGN KEY (poll_id) REFERENCES polls (id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create votes table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS votes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            poll_id INTEGER NOT NULL,
            option_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (poll_id) REFERENCES polls (id) ON DELETE CASCADE,
            FOREIGN KEY (option_id) REFERENCES poll_options (id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
            UNIQUE(poll_id, user_id) -- Each user can only vote once per poll
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create an admin user if no users exist
    let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    if user_count == 0 {
        // Create a default admin user (password: admin)
        let admin_password_hash = bcrypt::hash("admin", bcrypt::DEFAULT_COST).unwrap();

        sqlx::query(
            r#"
            INSERT INTO users (username, password_hash, is_admin)
            VALUES ('admin', ?, TRUE)
            "#,
        )
        .bind(admin_password_hash)
        .execute(pool)
        .await?;

        info!("Created default admin user (username: admin, password: admin)");
    }

    info!("Database initialization complete");
    Ok(())
}

// User database operations
pub async fn get_user_by_username(
    pool: &Pool<Sqlite>,
    username: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn get_user_by_id(pool: &Pool<Sqlite>, id: i64) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create_user(
    pool: &Pool<Sqlite>,
    username: &str,
    password_hash: &str,
    is_admin: bool,
) -> Result<User, sqlx::Error> {
    let id = sqlx::query(
        r#"
        INSERT INTO users (username, password_hash, created_at, is_admin)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(username)
    .bind(password_hash)
    .bind(Utc::now())
    .bind(is_admin)
    .execute(pool)
    .await?
    .last_insert_rowid();

    Ok(User {
        id,
        username: username.to_string(),
        password_hash: password_hash.to_string(),
        created_at: Utc::now(),
        is_admin,
    })
}

pub async fn list_users(pool: &Pool<Sqlite>) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY username")
        .fetch_all(pool)
        .await
}

// Poll database operations
pub async fn create_poll(
    pool: &Pool<Sqlite>,
    title: &str,
    description: Option<&str>,
    created_by: i64,
    expires_at: Option<DateTime<Utc>>,
) -> Result<i64, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let poll_id = sqlx::query(
        r#"
        INSERT INTO polls (title, description, created_by, created_at, expires_at, is_active)
        VALUES (?, ?, ?, ?, ?, TRUE)
        "#,
    )
    .bind(title)
    .bind(description)
    .bind(created_by)
    .bind(Utc::now())
    .bind(expires_at)
    .execute(&mut *transaction)
    .await?
    .last_insert_rowid();

    transaction.commit().await?;

    Ok(poll_id)
}

pub async fn add_poll_option(
    pool: &Pool<Sqlite>,
    poll_id: i64,
    text: &str,
    datetime_option: Option<DateTime<Utc>>,
) -> Result<i64, sqlx::Error> {
    let option_id = sqlx::query(
        r#"
        INSERT INTO poll_options (poll_id, text, datetime_option)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(poll_id)
    .bind(text)
    .bind(datetime_option)
    .execute(pool)
    .await?
    .last_insert_rowid();

    Ok(option_id)
}

pub async fn get_poll(pool: &Pool<Sqlite>, id: i64) -> Result<Option<Poll>, sqlx::Error> {
    sqlx::query_as::<_, Poll>("SELECT * FROM polls WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_poll_options(
    pool: &Pool<Sqlite>,
    poll_id: i64,
) -> Result<Vec<PollOption>, sqlx::Error> {
    sqlx::query_as::<_, PollOption>("SELECT * FROM poll_options WHERE poll_id = ?")
        .bind(poll_id)
        .fetch_all(pool)
        .await
}

pub async fn list_polls(pool: &Pool<Sqlite>, active_only: bool) -> Result<Vec<Poll>, sqlx::Error> {
    let query = if active_only {
        "SELECT * FROM polls WHERE is_active = TRUE ORDER BY created_at DESC"
    } else {
        "SELECT * FROM polls ORDER BY created_at DESC"
    };

    sqlx::query_as::<_, Poll>(query).fetch_all(pool).await
}

pub async fn vote(
    pool: &Pool<Sqlite>,
    poll_id: i64,
    option_id: i64,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;

    // Delete any existing vote by this user for this poll
    sqlx::query("DELETE FROM votes WHERE poll_id = ? AND user_id = ?")
        .bind(poll_id)
        .bind(user_id)
        .execute(&mut *transaction)
        .await?;

    // Insert the new vote
    sqlx::query(
        r#"
        INSERT INTO votes (poll_id, option_id, user_id, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(poll_id)
    .bind(option_id)
    .bind(user_id)
    .bind(Utc::now())
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
}

pub async fn get_user_vote(
    pool: &Pool<Sqlite>,
    poll_id: i64,
    user_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let result = sqlx::query_scalar::<_, i64>(
        "SELECT option_id FROM votes WHERE poll_id = ? AND user_id = ?",
    )
    .bind(poll_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

pub async fn get_vote_counts(
    pool: &Pool<Sqlite>,
    poll_id: i64,
) -> Result<Vec<(i64, i64)>, sqlx::Error> {
    // Return (option_id, count) pairs
    let results = sqlx::query_as::<_, (i64, i64)>(
        r#"
        SELECT option_id, COUNT(*) as vote_count
        FROM votes
        WHERE poll_id = ?
        GROUP BY option_id
        "#,
    )
    .bind(poll_id)
    .fetch_all(pool)
    .await?;

    Ok(results)
}

// Update poll status based on expiration date
pub async fn update_poll_statuses(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE polls
        SET is_active = FALSE
        WHERE expires_at IS NOT NULL AND expires_at < datetime('now')
        AND is_active = TRUE
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
