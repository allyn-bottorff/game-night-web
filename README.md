# Platform Engineering Game Night

A web application for creating and hosting polls for game nights. The application allows team members to log in, create polls, vote, and view results.

## Features

- User authentication (login, registration)
- Poll creation with arbitrary options
- Support for calendar dates and times as poll options
- Live poll results
- Expiration dates for polls
- Admin panel for user management
- Mobile-friendly interface

## Project Structure

The project is divided into two main parts:

### Backend

- Built with Rust using the Axum web framework
- SQLite database for persistent storage
- JWT-based authentication
- RESTful API endpoints
- Prometheus metrics

### Frontend

- Built with HTML5, CSS, and vanilla JavaScript
- Responsive design for mobile and desktop
- No external frameworks or libraries (except Font Awesome for icons)

## Getting Started

### Prerequisites

- Rust and Cargo
- SQLite

### Running the Backend

1. Navigate to the backend directory:
   ```
   cd game-night-web/backend
   ```

2. Run the server:
   ```
   cargo run
   ```

The server will start on `http://localhost:3000` by default.

### Running the Frontend

1. Navigate to the frontend directory:
   ```
   cd game-night-web/frontend
   ```

2. Serve the files with any simple HTTP server. For example, using Python:
   ```
   python -m http.server 8080
   ```

The frontend will be available at `http://localhost:8080`.

## API Endpoints

- `POST /api/login` - User login
- `POST /api/register` - User registration
- `GET /api/polls` - List polls
- `POST /api/polls` - Create a new poll
- `GET /api/polls/:id` - Get a specific poll
- `PUT /api/polls/:id` - Update a poll
- `POST /api/polls/:id/vote` - Vote on a poll
- `GET /api/polls/:id/results` - Get poll results
- `GET /api/users` - List users (admin only)
- `POST /api/users` - Create a new user (admin only)
- `GET /api/health` - Health check
- `GET /api/metrics` - Prometheus metrics

## Default Credentials

A default admin user is created on first run:
- Username: `admin`
- Password: `admin`

It's recommended to change this password after first login.