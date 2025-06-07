#!/bin/bash

# Script to reset the admin password in the game-night-web application

set -e

# Ensure we're in the project root directory
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$DIR"

# Default password is 'admin' if not provided
PASSWORD=${1:-admin}

# Load environment variables from .env file if it exists
if [ -f .env ]; then
    source <(grep -v '^#' .env | sed 's/^/export /')
fi

# Use default database path if not set in environment
DB_PATH=${DATABASE_URL:-"sqlite:./game_night.db"}
DB_PATH=${DB_PATH#sqlite:}

echo "Resetting admin password to '$PASSWORD'..."
echo "Using database at: $DB_PATH"

# Check if database file exists
if [ ! -f "$DB_PATH" ]; then
    echo "Database file not found. Creating a new one."
    # Create the users table if it doesn't exist
    sqlite3 "$DB_PATH" <<EOF
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
EOF
fi

# Generate bcrypt hash for the password
# We'll use Python for this since it's more widely available than specialized tools
HASH=$(python3 -c "import bcrypt; print(bcrypt.hashpw('$PASSWORD'.encode(), bcrypt.gensalt(12)).decode())")

echo "Generated password hash: $HASH"

# Check if admin user exists
ADMIN_EXISTS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM users WHERE username = 'admin';")

if [ "$ADMIN_EXISTS" -gt 0 ]; then
    # Update existing admin user
    sqlite3 "$DB_PATH" "UPDATE users SET password_hash = '$HASH' WHERE username = 'admin';"
    echo "✅ Admin password updated successfully"
else
    # Create admin user
    sqlite3 "$DB_PATH" "INSERT INTO users (username, password_hash, is_admin) VALUES ('admin', '$HASH', 1);"
    echo "✅ Admin user created successfully"
fi

echo "You can now log in with:"
echo "Username: admin"
echo "Password: $PASSWORD"