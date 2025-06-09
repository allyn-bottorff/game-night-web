# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
```bash
# Run the application (must be run from src/)
cargo run

# Build for production
cargo build --release

# Run tests
cargo test
```

### Database Management
```bash
# Reset admin password to 'admin'
python3 reset_admin_password.py

# Set custom admin password
python3 reset_admin_password.py your_new_password

# Alternative bash script
./reset_admin_password.sh [password]
```

### Examples
```bash
# Hash a password for testing
cargo run --example hash_password

# Reset admin password via Rust
cargo run --example reset_admin_password

# Test database initialization
cargo run --example test_db_init

# Test startup
cargo run --example test_startup
```

## Architecture

### MVC Structure
- **Models** (`src/models/`): Data structures and database operations
  - `user.rs`: User authentication, password hashing, admin management
  - `poll.rs`: Poll creation, voting logic, expiration handling
- **Controllers** (`src/controllers/`): Business logic layer
  - `users.rs`: User management, authentication logic
  - `polls.rs`: Poll CRUD operations, voting mechanics
  - `metrics.rs`: Prometheus metrics collection
- **Routes** (`src/routes/mod.rs`): HTTP endpoint definitions and request handling

### Database Layer
- SQLite with SQLx for async database operations
- Connection pooling via `SqlitePool` (max 5 connections)
- Database wrapper `DbConn` implements Rocket's `FromRequest` for dependency injection
- Automatic migrations on startup from `migrations/` directory
- Auto-creates default admin user (admin/admin) if no admin exists

### Authentication System
- Session-based authentication using Rocket's private cookies
- Password hashing with bcrypt
- Role-based access control (admin vs regular users)
- Admin users can manage other users and have enhanced poll management capabilities

### Template Engine
- Uses Tera templates (not Handlebars)
- Templates in `src/templates/` directory
- Avoid ternary operators - use `{% if %}` blocks instead
- Static assets served from `src/static/`

### Key Configuration
- Environment variables loaded from `.env` file
- `DATABASE_URL` defaults to `sqlite:game_night.db`
- `ROCKET_SECRET_KEY` required for session management
- Application runs on port 8000 by default

### Poll System Architecture
- Polls have expiration dates and support multiple voting options
- Options can be text-based or date/time selections
- Users can vote on multiple options per poll
- Poll creators and admins can delete polls
- Vote tracking prevents duplicate votes per user/option pair

### Metrics
- Prometheus-compatible metrics exposed at `/metrics` endpoint
- Tracks application performance and usage statistics
