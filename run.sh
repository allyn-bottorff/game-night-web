#!/bin/bash

# Ensure we're in the project root directory
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$DIR"

# Make scripts executable
chmod +x reset_admin_password.sh || true

# Check if .env file exists, create it if it doesn't
if [ ! -f .env ]; then
    echo "Creating default .env file..."
    cat > .env << EOL
DATABASE_URL=sqlite:./game_night.db
ROCKET_SECRET_KEY=qiAuQbzIpvnFT23klCTz6+qMW8nqUk52LG6FPrAJkP4=
ROCKET_ADDRESS=127.0.0.1
ROCKET_PORT=8000
ROCKET_LOG=normal
RUST_LOG=info
EOL
fi

# Check if reset scripts exist
if [ ! -f "reset_admin_password.py" ] && [ ! -f "reset_admin_password.sh" ]; then
    echo "Warning: Password reset scripts not found. Default login might not work."
fi

# Reset admin password using Python script if available
if command -v python3 &> /dev/null && python3 -c "import bcrypt" &> /dev/null; then
    echo "Resetting admin password using Python script..."
    python3 reset_admin_password.py
elif command -v bash &> /dev/null && command -v sqlite3 &> /dev/null; then
    echo "Resetting admin password using Bash script..."
    bash reset_admin_password.sh
else
    echo "Warning: Could not reset admin password. Please run reset_admin_password.py or reset_admin_password.sh manually."
    echo "Default login credentials: username='admin', password='admin'"
fi

# Run the application
echo "Starting Game Night Web Application..."
cargo run