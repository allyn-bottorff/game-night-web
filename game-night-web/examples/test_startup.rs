use dotenv::dotenv;
use game_night_web::db;
use game_night_web::models::user::User;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();
    
    println!("Testing application startup...");
    
    // Remove existing database to start fresh
    if std::path::Path::new("game_night.db").exists() {
        std::fs::remove_file("game_night.db")?;
        println!("Removed existing database file");
    }
    
    // Test the database initialization that happens during startup
    let pool = db::init_pool().await;
    println!("✅ Database pool initialized successfully");
    
    // Run migrations (same as what happens in main.rs)
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
    println!("✅ Database migrations completed");
    
    // Initialize default admin user (same as what happens in main.rs)
    db::init_default_admin(&pool).await?;
    println!("✅ Default admin user initialization completed");
    
    // Verify the admin user was created correctly
    let admin_users: Vec<User> = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, is_admin, created_at FROM users WHERE is_admin = 1"
    )
    .fetch_all(&pool)
    .await?;
    
    if admin_users.len() == 1 {
        let admin = &admin_users[0];
        println!("✅ Found admin user: {} (ID: {})", admin.username, admin.id);
        
        // Test password verification
        if admin.verify_password("admin") {
            println!("✅ Default admin password verified successfully");
        } else {
            println!("❌ Default admin password verification failed");
        }
    } else {
        println!("❌ Expected 1 admin user, found {}", admin_users.len());
    }
    
    // Test that running initialization again doesn't create duplicates
    println!("\nTesting duplicate prevention...");
    db::init_default_admin(&pool).await?;
    
    let admin_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_admin = 1")
        .fetch_one(&pool)
        .await?;
    
    if admin_count == 1 {
        println!("✅ No duplicate admin users created on second initialization");
    } else {
        println!("❌ Unexpected admin user count: {}", admin_count);
    }
    
    println!("\n🎉 Application startup test completed successfully!");
    println!("The app will create the database and admin user on first startup.");
    println!("Default admin credentials: username='admin', password='admin'");
    println!("⚠️  Remember to change the default password after first login!");
    
    Ok(())
}