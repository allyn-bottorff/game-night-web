# Platform Engineering Game Night - Coding Instructions

## Project Overview
Develop a web application for "Platform Engineering Game Night" that allows team members to create and participate in polls for scheduling game nights. The application will feature user authentication, poll creation/management, and real-time results viewing.

## Technology Stack
- **Framework**: Rust with Rocket framework (latest version) as an MVC framework
- **Database**: SQLite
- **Authentication**: Custom auth system using sessions/cookies
- **Frontend**: HTML, CSS, JavaScript served by Rocket using templates

## Application Requirements

### System Architecture
1. Create a single-instance web application using Rocket's MVC architecture
2. Structure the project with clear separation between routes, models, views (templates), and controllers
3. Use Rocket's template engine for rendering HTML views
4. Implement proper error handling throughout the application

### Authentication & User Management
1. Implement a user authentication system with login/logout functionality
2. No anonymous access to any part of the application
3. User roles:
   - Regular users: Can create and vote in polls
   - Admin users: Can add new users to the system
4. Store user credentials securely in the SQLite database

### Poll Management
1. Create data models for polls with the following attributes:
   - Title and description
   - Multiple options (variable number)
   - Creator information
   - Creation timestamp
   - Expiration date/time
   - Current status (active/expired)
   - Votes
2. Implement API endpoints for:
   - Creating new polls
   - Listing active polls
   - Listing inactive/expired polls
   - Voting on polls
   - Viewing poll results
3. Support for calendar date/time options in polls

### Frontend Features
1. Create Rocket templates for rendering responsive web interfaces including:
   - Login page
   - Dashboard showing active polls
   - Section for viewing inactive/expired polls
   - Poll creation form
   - Poll voting interface
   - Results visualization
   - User management interface (for admins)
2. Use Rocket's template context to pass data from controllers to views
3. Implement real-time or regularly updating poll results
4. Create clear visual distinction between active and expired polls

### Database Integration
1. Design and implement SQLite database schema for:
   - Users
   - Polls
   - Options
   - Votes
2. Use appropriate Rust crates for SQLite integration
3. Implement proper database migrations

### Monitoring
1. Create a Prometheus-compatible metrics endpoint
2. Track key metrics such as:
   - Number of active polls
   - Number of votes
   - API endpoint usage
   - Authentication attempts

## Implementation Guidelines
1. Use native Rust implementations where feasible, minimizing external dependencies
2. Utilize Rocket's MVC features including templates, forms, and guards
3. Follow best practices for Rust code organization
4. Implement proper error handling and logging
5. Write clear, concise code with appropriate comments
6. Include basic unit and integration tests
7. Provide documentation for setting up and running the application

## Project Structure (Suggested)
```
game-night-web/
├── src/
│   ├── main.rs              # Application entry point
│   ├── models/              # Data models
│   ├── controllers/         # Business logic
│   ├── routes/              # Route definitions
│   ├── db/                  # Database operations
│   ├── auth/                # Authentication logic
│   ├── templates/           # Rocket template files (views)
│   └── static/              # Static assets (CSS, JS)
├── migrations/              # Database migrations
├── tests/                   # Tests
├── Cargo.toml               # Dependencies
└── README.md                # Project documentation
```

## Deliverables
1. Complete source code for the web application
2. Database schema and migration scripts
3. Setup and usage documentation
4. Basic test suite

## Extra Considerations
1. Ensure all user inputs are properly validated
2. Implement CSRF protection
3. Consider rate limiting for authentication attempts
4. Make the UI intuitive and easy to use
5. Consider adding email notifications for poll creation/expiration (optional)