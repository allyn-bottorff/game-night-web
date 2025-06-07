use bcrypt;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: cargo run --example hash_password <password>");
        return;
    }
    
    let password = &args[1];
    
    match bcrypt::hash(password, 12) {
        Ok(hash) => println!("Bcrypt hash for '{}': {}", password, hash),
        Err(e) => println!("Error generating hash: {}", e),
    }
}