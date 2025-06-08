// use log::error;
// use sqlx::Row;
// use sqlx::sqlite::SqlitePool;

// Database table creation statements
// pub const CREATE_USERS_TABLE: &str = r#"
// CREATE TABLE IF NOT EXISTS users (
//     id INTEGER PRIMARY KEY AUTOINCREMENT,
//     username TEXT NOT NULL UNIQUE,
//     password_hash TEXT NOT NULL,
//     is_admin BOOLEAN NOT NULL DEFAULT 0,
//     created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
// )
// "#;

// pub const CREATE_POLLS_TABLE: &str = r#"
// CREATE TABLE IF NOT EXISTS polls (
//     id INTEGER PRIMARY KEY AUTOINCREMENT,
//     title TEXT NOT NULL,
//     description TEXT,
//     creator_id INTEGER NOT NULL,
//     created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
//     expires_at TIMESTAMP NOT NULL,
//     FOREIGN KEY (creator_id) REFERENCES users(id) ON DELETE CASCADE
// )
// "#;

// pub const CREATE_OPTIONS_TABLE: &str = r#"
// CREATE TABLE IF NOT EXISTS options (
//     id INTEGER PRIMARY KEY AUTOINCREMENT,
//     poll_id INTEGER NOT NULL,
//     text TEXT NOT NULL,
//     is_date BOOLEAN NOT NULL DEFAULT 0,
//     date_time TIMESTAMP,
//     FOREIGN KEY (poll_id) REFERENCES polls(id) ON DELETE CASCADE
// )
// "#;

// pub const CREATE_VOTES_TABLE: &str = r#"
// CREATE TABLE IF NOT EXISTS votes (
//     id INTEGER PRIMARY KEY AUTOINCREMENT,
//     user_id INTEGER NOT NULL,
//     option_id INTEGER NOT NULL,
//     created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
//     FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
//     FOREIGN KEY (option_id) REFERENCES options(id) ON DELETE CASCADE,
//     UNIQUE(user_id, option_id)
// )
// "#;

// // Initialize the database schema
// pub async fn initialize_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
//     let mut tx = pool.begin().await?;

//     sqlx::query(CREATE_USERS_TABLE)
//         .execute(&mut *tx)
//         .await
//         .map_err(|e| {
//             error!("Failed to create users table: {}", e);
//             e
//         })?;

//     sqlx::query(CREATE_POLLS_TABLE)
//         .execute(&mut *tx)
//         .await
//         .map_err(|e| {
//             error!("Failed to create polls table: {}", e);
//             e
//         })?;

//     sqlx::query(CREATE_OPTIONS_TABLE)
//         .execute(&mut *tx)
//         .await
//         .map_err(|e| {
//             error!("Failed to create options table: {}", e);
//             e
//         })?;

//     sqlx::query(CREATE_VOTES_TABLE)
//         .execute(&mut *tx)
//         .await
//         .map_err(|e| {
//             error!("Failed to create votes table: {}", e);
//             e
//         })?;

//     tx.commit().await?;
//     Ok(())
// }

// Create an initial admin user if no users exist
// pub async fn create_initial_admin(
//     pool: &SqlitePool,
//     username: &str,
//     password_hash: &str,
// ) -> Result<(), sqlx::Error> {
//     // Check if any users exist
//     let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM users")
//         .fetch_one(pool)
//         .await?
//         .get("count");

//     if count == 0 {
//         // Create admin user
//         sqlx::query("INSERT INTO users (username, password_hash, is_admin) VALUES (?, ?, 1)")
//             .bind(username)
//             .bind(password_hash)
//             .execute(pool)
//             .await?;
//     }

//     Ok(())
// }
