use bcrypt;
use sqlx::sqlite::SqlitePool;
use std::env;
use dotenv::dotenv;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let args: Vec<String> = env::args().collect();
    let password = if args.len() >= 2 {
        &args[1]
    } else {
        "admin" // Default password
    };
    
    println!("Resetting admin password to '{}'...", password);
    
    // Generate password hash
    let password_hash = bcrypt::hash(password, 12)?;
    println!("Generated hash: {}", password_hash);
    
    // Connect to the database
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./game_night.db".to_string());
    println!("Connecting to database at: {}", database_url);
    
    let pool = SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new()
            .filename(database_url.strip_prefix("sqlite:").unwrap_or("./game_night.db"))
            .create_if_missing(true)
    ).await?;
    
    // Check if admin user exists
    let admin_exists = sqlx::query!("SELECT COUNT(*) as count FROM users WHERE username = 'admin'")
        .fetch_one(&pool)
        .await?
        .count > 0;
    
    if admin_exists {
        // Update existing admin password
        sqlx::query!(
            "UPDATE users SET password_hash = ? WHERE username = 'admin'",
            password_hash
        )
        .execute(&pool)
        .await?;
        
        println!("✅ Admin password updated successfully");
    } else {
        // Create admin user
        sqlx::query!(
            "INSERT INTO users (username, password_hash, is_admin) VALUES ('admin', ?, 1)",
            password_hash
        )
        .execute(&pool)
        .await?;
        
        println!("✅ Admin user created successfully");
    }
    
    Ok(())
}