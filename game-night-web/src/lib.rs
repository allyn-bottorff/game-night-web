//! # Game Night Web Application Library
//!
//! This library provides the core modules for the Game Night web application,
//! including authentication, database operations, models, controllers, and routing.
//!
//! ## Modules
//! - [`auth`] - Authentication and authorization system
//! - [`controllers`] - Business logic layer for handling requests
//! - [`db`] - Database connection and operations
//! - [`models`] - Data structures and models
//! - [`routes`] - HTTP route definitions and handlers

/// Authentication and authorization module providing user login/logout,
/// session management, and role-based access control.
pub mod auth;

/// Controllers module containing business logic for handling HTTP requests
/// and coordinating between routes and database operations.
pub mod controllers;

/// Database module providing connection pooling, migrations, and data access operations.
pub mod db;

/// Models module defining data structures, forms, and database entity representations.
pub mod models;

/// Routes module defining HTTP endpoints and request handlers for the web application.
pub mod routes;