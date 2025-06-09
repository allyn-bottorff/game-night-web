//! # Controllers Module
//!
//! This module contains the business logic layer for the Game Night application.
//! Controllers handle the processing of HTTP requests, coordinate with the database,
//! and prepare data for the presentation layer.
//!
//! ## Submodules
//! - [`polls`] - Poll management, voting, and statistics
//! - [`users`] - User management, authentication, and roles
//!
//! ## Architecture
//! Controllers follow the MVC pattern by:
//! - Receiving data from route handlers
//! - Validating and processing business logic
//! - Interacting with the database layer
//! - Returning formatted responses for the view layer

/// Poll-related business logic including creation, voting, deletion, and statistics.
pub mod polls;

/// User-related business logic including authentication, management, and roles.
pub mod users;
