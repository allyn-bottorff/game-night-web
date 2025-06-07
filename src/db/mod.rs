use rocket::fairing::AdHoc;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::env;
use std::time::Duration;

pub mod schema;

pub struct DbConn(pub sqlx::pool::PoolConnection<sqlx::Sqlite>);

/// Initialize the SQLite database
pub async fn init_pool() -> SqlitePool {
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:game_night.db".to_string());

    SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("Failed to connect to SQLite database")
}

/// Database initialization hook for Rocket
pub fn init_db() -> AdHoc {
    AdHoc::try_on_ignite("SQLite Database", |rocket| async {
        let pool = init_pool().await;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run database migrations");

        Ok(rocket.manage(pool))
    })
}

/// Retrieve a database connection from the managed pool
pub async fn get_conn(pool: &SqlitePool) -> Result<DbConn, sqlx::Error> {
    let conn = pool.acquire().await?;
    Ok(DbConn(conn))
}

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
