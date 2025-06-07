use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use rocket::form::FromForm;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Poll {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub creator_id: i64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PollWithCreator {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub creator_id: i64,
    pub creator_username: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PollOption {
    pub id: i64,
    pub poll_id: i64,
    pub text: String,
    pub is_date: bool,
    pub date_time: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub vote_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vote {
    pub id: i64,
    pub user_id: i64,
    pub option_id: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromForm, Deserialize)]
pub struct NewPollForm {
    pub title: String,
    pub description: Option<String>,
    pub expires_at: String, // Format: YYYY-MM-DDTHH:MM
    pub options: String,    // Comma-separated options
}

#[derive(Debug, FromForm, Deserialize)]
pub struct VoteForm {
    pub option_id: i64,
}

impl Poll {
    pub fn is_active(&self) -> bool {
        self.expires_at > Utc::now()
    }

    pub fn get_status(&self) -> &'static str {
        if self.is_active() {
            "active"
        } else {
            "expired"
        }
    }
}