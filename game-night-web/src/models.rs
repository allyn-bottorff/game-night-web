//! # Data Models and Structures
//!
//! This module contains all data models, forms, and structures used throughout
//! the Game Night application. It includes user models, poll models, voting models,
//! and common utilities like flash messages.
//!
//! ## Key Components
//! - User authentication and management models
//! - Poll creation and voting system models
//! - Form structures for HTTP requests
//! - Common utilities like flash messages

use chrono::{DateTime, Utc};
use rocket::form::FromForm;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ============================================================================
// Common structures and enums
// ============================================================================
/// Represents a flash message displayed to users for feedback.
/// Used to show success, error, warning, or informational messages.
#[derive(Debug, Serialize, Deserialize)]
pub struct FlashMessage {
    /// The type/category of the message (success, error, warning, info)
    pub message_type: MessageType,
    /// The actual message text to display to the user
    pub message: String,
}

/// Enumeration of different message types for user feedback.
/// Used to determine the styling and context of flash messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Indicates a successful operation (typically shown in green)
    Success,
    /// Provides informational content (typically shown in blue)
    Info,
    /// Warns about potential issues (typically shown in yellow/orange)
    Warning,
    /// Indicates an error or failure (typically shown in red)
    Error,
}

impl ToString for MessageType {
    /// Converts the MessageType enum to a string representation.
    /// Used for CSS class names and template rendering.
    ///
    /// # Returns
    /// A string representation of the message type
    fn to_string(&self) -> String {
        match self {
            MessageType::Success => "success".to_string(),
            MessageType::Info => "info".to_string(),
            MessageType::Warning => "warning".to_string(),
            MessageType::Error => "error".to_string(),
        }
    }
}

// ============================================================================
// User-related models
// ============================================================================
/// Represents a user in the system with authentication and role information.
/// This is the core user model stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    /// Unique identifier for the user
    pub id: i64,
    /// Unique username for authentication
    pub username: String,
    /// Bcrypt-hashed password (excluded from serialization for security)
    #[serde(skip_serializing)]
    pub password_hash: String,
    /// Whether the user has administrative privileges
    pub is_admin: bool,
    /// Timestamp when the user account was created
    pub created_at: DateTime<Utc>,
}

/// Form data structure for user login requests.
/// Captures username and password from the login form.
#[derive(Debug, FromForm, Deserialize)]
pub struct LoginForm {
    /// Username entered by the user
    pub username: String,
    /// Plain text password (will be verified against stored hash)
    pub password: String,
}

/// Form data structure for creating new user accounts.
/// Used by administrators to add new users to the system.
#[derive(Debug, FromForm, Deserialize)]
pub struct NewUserForm {
    /// Desired username for the new account
    pub username: String,
    /// Plain text password for the new account
    pub password: String,
    /// Password confirmation to prevent typos
    pub confirm_password: String,
    /// Whether the new user should have admin privileges
    pub is_admin: bool,
}

/// Form data structure for password change requests.
/// Allows users to update their password with current password verification.
#[derive(Debug, FromForm, Deserialize)]
pub struct ChangePasswordForm {
    /// Current password for verification
    pub current_password: String,
    /// New password to set
    pub new_password: String,
    /// Confirmation of the new password
    pub confirm_password: String,
}

/// Form data structure for changing user roles.
/// Used by administrators to promote/demote users to/from admin status.
#[derive(Debug, FromForm, Deserialize)]
pub struct ToggleRoleForm {
    /// ID of the user whose role should be changed
    pub user_id: i64,
    /// Whether to set admin privileges (true) or remove them (false)
    pub set_admin: bool,
}

impl User {
    /// Verifies a plain text password against the user's stored password hash.
    ///
    /// # Arguments
    /// * `password` - The plain text password to verify
    ///
    /// # Returns
    /// `true` if the password matches the stored hash, `false` otherwise
    pub fn verify_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.password_hash).unwrap_or(false)
    }

    /// Hashes a plain text password using bcrypt with cost factor 12.
    ///
    /// # Arguments
    /// * `password` - The plain text password to hash
    ///
    /// # Returns
    /// `Ok(String)` containing the hashed password, or `Err` if hashing fails
    pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
        bcrypt::hash(password, 12)
    }
}

// ============================================================================
// Poll-related models
// ============================================================================
/// Represents a poll in the system.
/// Polls are created by users and contain multiple options that can be voted on.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Poll {
    /// Unique identifier for the poll
    pub id: i64,
    /// The poll's title/question
    pub title: String,
    /// Optional detailed description of the poll
    pub description: Option<String>,
    /// ID of the user who created this poll
    pub creator_id: i64,
    /// Timestamp when the poll was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the poll expires and voting closes
    pub expires_at: DateTime<Utc>,
}

/// Extended poll information that includes the creator's username.
/// Used for displaying polls with creator information in templates.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PollWithCreator {
    /// Unique identifier for the poll
    pub id: i64,
    /// The poll's title/question
    pub title: String,
    /// Optional detailed description of the poll
    pub description: Option<String>,
    /// ID of the user who created this poll
    pub creator_id: i64,
    /// Username of the poll creator
    pub creator_username: String,
    /// Timestamp when the poll was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the poll expires and voting closes
    pub expires_at: DateTime<Utc>,
}

/// Represents a voting option within a poll.
/// Options can be either text-based or date/time selections.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PollOption {
    /// Unique identifier for the option
    pub id: i64,
    /// ID of the poll this option belongs to
    pub poll_id: i64,
    /// Text description of the option
    pub text: String,
    /// Whether this option represents a date/time choice
    pub is_date: bool,
    /// Optional date/time value for date-based options
    pub date_time: Option<DateTime<Utc>>,
    /// Number of votes this option has received (calculated field)
    #[sqlx(default)]
    pub vote_count: i64,
}

/// Represents a user's vote on a specific poll option.
/// Each vote links a user to a poll option with a timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vote {
    /// Unique identifier for the vote
    pub id: i64,
    /// ID of the user who cast this vote
    pub user_id: i64,
    /// ID of the poll option that was voted for
    pub option_id: i64,
    /// Timestamp when the vote was cast
    pub created_at: DateTime<Utc>,
}

/// Form data structure for creating new polls.
/// Captures all necessary information to create a poll with options.
#[derive(Debug, FromForm, Deserialize)]
pub struct NewPollForm {
    /// Title/question for the poll
    pub title: String,
    /// Optional detailed description
    pub description: Option<String>,
    /// Expiration date/time in format YYYY-MM-DDTHH:MM
    pub expires_at: String,
    /// Comma-separated list of poll options
    pub options: String,
}

/// Form data structure for creating new poll options.
#[derive(Debug, FromForm, Deserialize)]
pub struct NewOptionsForm {
    /// Comma-separated list of poll options
    pub options: String,
}

/// Form data structure for casting votes on poll options.
/// Simple form containing only the option ID being voted for.
#[derive(Debug, FromForm, Deserialize)]
pub struct VoteForm {
    /// ID of the poll option to vote for
    pub option_id: i64,
}

/// Extended vote information that includes the voter's username.
/// Used for displaying detailed voting information with user context.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VoteWithUser {
    /// Unique identifier for the vote
    pub vote_id: i64,
    /// ID of the user who cast the vote
    pub user_id: i64,
    /// Username of the voter
    pub username: String,
    /// ID of the poll option that was voted for
    pub option_id: i64,
    /// Timestamp when the vote was cast
    pub created_at: DateTime<Utc>,
}

/// Poll option with detailed voter information.
/// Combines option details with a list of all users who voted for it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionWithVoters {
    /// Unique identifier for the option
    pub id: i64,
    /// ID of the poll this option belongs to
    pub poll_id: i64,
    /// Text description of the option
    pub text: String,
    /// Whether this option represents a date/time choice
    pub is_date: bool,
    /// Optional date/time value for date-based options
    pub date_time: Option<DateTime<Utc>>,
    /// Total number of votes for this option
    pub vote_count: i64,
    /// Detailed list of all votes cast for this option
    pub voters: Vec<VoteWithUser>,
}

/// Complete poll information including voting details and statistics.
/// Used for detailed poll views showing all options, votes, and voter information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollVotingDetails {
    /// The poll information including creator details
    pub poll: PollWithCreator,
    /// All poll options with their respective voters
    pub options_with_voters: Vec<OptionWithVoters>,
    /// Total number of votes cast across all options
    pub total_votes: i64,
    /// Total number of unique voters who participated
    pub total_voters: i64,
}

// impl Poll {
//     pub fn is_active(&self) -> bool {
//         self.expires_at > Utc::now()
//     }

//     pub fn get_status(&self) -> &'static str {
//         if self.is_active() {
//             "active"
//         } else {
//             "expired"
//         }
//     }
// }

