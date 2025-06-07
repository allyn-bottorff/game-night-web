import os
import sys
import sqlite3
import bcrypt
from pathlib import Path
import re

def main():
    # Get password from command line args or use default
    password = sys.argv[1] if len(sys.argv) > 1 else 'admin'
    print(f"Resetting admin password to '{password}'...")
    
    # Find project root directory (where this script is located)
    project_root = Path(__file__).parent.absolute()
    os.chdir(project_root)
    
    # Load environment variables from .env if it exists
    env_vars = {}
    env_path = project_root / '.env'
    if env_path.exists():
        with open(env_path, 'r') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#'):
                    key, value = line.split('=', 1)
                    env_vars[key] = value
    
    # Get database path
    db_url = env_vars.get('DATABASE_URL', 'sqlite:./game_night.db')
    db_path = db_url.replace('sqlite:', '')
    
    print(f"Using database at: {db_path}")
    
    # Create directory for database if it doesn't exist
    db_dir = os.path.dirname(db_path)
    if db_dir and not os.path.exists(db_dir):
        os.makedirs(db_dir)
    
    # Connect to database
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # Create users table if it doesn't exist
    cursor.execute('''
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        username TEXT NOT NULL UNIQUE,
        password_hash TEXT NOT NULL,
        is_admin BOOLEAN NOT NULL DEFAULT 0,
        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );
    ''')
    
    # Generate bcrypt hash
    password_hash = bcrypt.hashpw(password.encode(), bcrypt.gensalt(12)).decode()
    print(f"Generated password hash: {password_hash}")
    
    # Check if admin user exists
    cursor.execute("SELECT COUNT(*) FROM users WHERE username = 'admin'")
    admin_exists = cursor.fetchone()[0] > 0
    
    if admin_exists:
        # Update existing admin user
        cursor.execute("UPDATE users SET password_hash = ? WHERE username = 'admin'", (password_hash,))
        print("✅ Admin password updated successfully")
    else:
        # Create admin user
        cursor.execute(
            "INSERT INTO users (username, password_hash, is_admin) VALUES (?, ?, 1)",
            ('admin', password_hash)
        )
        print("✅ Admin user created successfully")
    
    # Commit changes and close connection
    conn.commit()
    conn.close()
    
    print("\nYou can now log in with:")
    print("Username: admin")
    print(f"Password: {password}")

if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"Error: {e}")
        print("\nYou may need to install required packages:")
        print("pip install bcrypt")
        sys.exit(1)