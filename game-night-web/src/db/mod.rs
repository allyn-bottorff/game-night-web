use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::env;
use std::time::Duration;
use crate::models::user::User;

pub mod schema;

pub struct DbConn(pub sqlx::pool::PoolConnection<sqlx::Sqlite>);

/// Initialize the SQLite database
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

/// Initialize database with default admin user if no admin exists
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
