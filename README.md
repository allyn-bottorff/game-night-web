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
# Important: Make sure to run this command from the project root directory, not from src/
cargo run
```

Or use the provided run script which also resets the admin password automatically:

```bash
./run.sh
```

The run script will attempt to use either the Python or Bash reset script, depending on what's available on your system.

6. Open your browser and navigate to `http://localhost:8000`

## Troubleshooting

### Database Connection Issues

If you encounter an error about not being able to open the database file, ensure:

1. You're running the application from the project root directory, not from inside the `src/` directory
2. The DATABASE_URL in your .env file is correctly set to `sqlite:./game_night.db`
3. The directory where the application is running has write permissions

### Template Engine Issues

This project uses Tera as its template engine. If you encounter template parsing errors:

1. Avoid using the ternary operator (`? :`) in templates, as it's not supported in Tera
2. Instead of `value | ternary('a', 'b')`, use:
   ```
   {% if value %}a{% else %}b{% endif %}
   ```
3. The `round` filter doesn't accept parameters in Tera (use `| round` instead of `| round(1)`)
4. For conditional classes, use:
   ```
   <div class="base-class {% if condition %}extra-class{% endif %}">
   ```

## Default Admin Account

The application is initialized with a default admin account:

- Username: `admin`
- Password: `admin`

**Important**: Please change the default admin password after the first login for security reasons.

### Resetting Admin Password

If you need to reset the admin password, you can use one of the provided scripts:

#### Using Python (Recommended)

```bash
# Reset to default 'admin' password
python3 reset_admin_password.py

# Or set a custom password
python3 reset_admin_password.py your_new_password
```

Requirements:
- Python 3
- bcrypt package (`pip install bcrypt`)

#### Using Bash

```bash
# Reset to default 'admin' password
./reset_admin_password.sh

# Or set a custom password
./reset_admin_password.sh your_new_password
```

Requirements:
- sqlite3 command-line tool
- Python 3 with bcrypt (for password hashing)

These scripts will update the password for the existing admin user or create the admin user if it doesn't exist.

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
# Must be run from the project root directory
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