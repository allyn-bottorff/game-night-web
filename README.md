# Platform Engineering Game Night Web Application

A web application for creating and managing polls for game night events. This application is built using Rust with the Rocket framework as an MVC application.

## Features

- User authentication and authorization
- Poll creation with multiple options
- Support for date/time options in polls
- Real-time poll results
- User management for administrators
- Prometheus-compatible metrics endpoint

## Prerequisites

- Rust and Cargo (latest stable version)
- SQLite

## Setup and Installation

1. Clone the repository:

```bash
git clone https://github.com/your-username/game-night-web.git
cd game-night-web
```

2. Copy the `.env.example` file to `.env` (or use the existing `.env` file):

```bash
cp .env.example .env
```

3. Generate a secure key for the Rocket secret:

```bash
openssl rand -base64 32
```

4. Update the `ROCKET_SECRET_KEY` in the `.env` file with the generated key.

5. Build and run the application:

```bash
cargo run
```

6. Open your browser and navigate to `http://localhost:8000`

## Default Admin Account

The application is initialized with a default admin account:

- Username: `admin`
- Password: `admin`

**Important**: Please change the default admin password after the first login for security reasons.

## Project Structure

- `src/`
  - `main.rs` - Application entry point
  - `models/` - Data models
  - `controllers/` - Business logic
  - `routes/` - Route definitions
  - `db/` - Database operations
  - `auth/` - Authentication logic
  - `templates/` - Rocket template files (views)
  - `static/` - Static assets (CSS, JS)
- `migrations/` - Database migrations
- `tests/` - Tests

## Database

The application uses SQLite as its database. The database file is created automatically when the application is first run. Database migrations are applied automatically during application startup.

## Metrics

The application exposes a Prometheus-compatible metrics endpoint at `/metrics` which can be scraped by Prometheus for monitoring.

## Development

### Running in Development Mode

```bash
cargo run
```

### Running Tests

```bash
cargo test
```

### Building for Production

```bash
cargo build --release
```

The compiled binary will be available at `target/release/game-night-web`.

## License

This project is licensed under the MIT License - see the LICENSE file for details.