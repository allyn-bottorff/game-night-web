use dotenv::dotenv;
use game_night_web::db;
use game_night_web::models::user::User;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();
    
    println!("Testing database initialization...");
    
    // Remove existing database to start fresh
    if std::path::Path::new("game_night.db").exists() {
        std::fs::remove_file("game_night.db")?;
        println!("Removed existing database file");
    }
    
    // Initialize database pool
    let pool = db::init_pool().await;
    println!("Database pool initialized");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
    println!("Migrations completed");
    
    // Initialize default admin user
    db::init_default_admin(&pool).await?;
    println!("Admin initialization completed");
    
    // Verify admin user was created
    let admin_users: Vec<User> = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, is_admin, created_at FROM users WHERE is_admin = 1"
    )
    .fetch_all(&pool)
    .await?;
    
    println!("Found {} admin user(s):", admin_users.len());
    for user in &admin_users {
        println!("  - Username: {}, ID: {}, Created: {}", 
                 user.username, user.id, user.created_at);
    }
    
    // Test login with default credentials
    if let Some(admin) = admin_users.first() {
        let password_valid = admin.verify_password("admin");
        println!("Default password verification: {}", if password_valid { "✅ PASS" } else { "❌ FAIL" });
    }
    
    // Test running initialization again (should not create duplicate)
    println!("\nTesting second initialization (should skip admin creation)...");
    db::init_default_admin(&pool).await?;
    
    let admin_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_admin = 1")
        .fetch_one(&pool)
        .await?;
    
    println!("Admin count after second init: {}", admin_count);
    
    if admin_count == 1 {
        println!("✅ SUCCESS: No duplicate admin users created");
    } else {
        println!("❌ FAILURE: Unexpected number of admin users");
    }
    
    println!("\nDatabase initialization test completed!");
    Ok(())
}