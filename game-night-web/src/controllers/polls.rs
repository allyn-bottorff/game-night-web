//! # Poll Controller Module
//!
//! This module contains all business logic related to poll management,
//! including poll creation, voting, deletion, and statistics.
//!
//! ## Key Functions
//! - Poll creation with options and expiration dates
//! - Voting and vote toggling functionality
//! - Poll deletion (by creator or admin)
//! - Poll querying (active, expired, detailed views)
//! - Voter statistics and detailed voting information
//! - Template data formatting

use chrono::Utc;
use log::{error, info};
use sqlx::{Row, SqlitePool};

use crate::models::{NewPollForm, PollOption, PollWithCreator, OptionWithVoters, VoteWithUser, PollVotingDetails, User};

/// Retrieves all active (non-expired) polls from the database.
/// 
/// This function queries for polls that have not yet reached their
/// expiration date, ordered by creation date (most recent first).
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// 
/// # Returns
/// * `Ok(Vec<PollWithCreator>)` - Vector of active polls with creator information
/// * `Err(sqlx::Error)` - Database error if query fails
pub async fn get_active_polls(pool: &SqlitePool) -> Result<Vec<PollWithCreator>, sqlx::Error> {
    sqlx::query_as::<_, PollWithCreator>(
        "SELECT p.id, p.title, p.description, p.creator_id, u.username as creator_username,
         p.created_at, p.expires_at
         FROM polls p
         JOIN users u ON p.creator_id = u.id
         WHERE p.expires_at > datetime('now')
         ORDER BY p.created_at DESC",
    )
    .fetch_all(pool)
    .await
}

/// Retrieves all expired polls from the database.
/// 
/// This function queries for polls that have passed their expiration
/// date, ordered by creation date (most recent first).
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// 
/// # Returns
/// * `Ok(Vec<PollWithCreator>)` - Vector of expired polls with creator information
/// * `Err(sqlx::Error)` - Database error if query fails
pub async fn get_expired_polls(pool: &SqlitePool) -> Result<Vec<PollWithCreator>, sqlx::Error> {
    sqlx::query_as::<_, PollWithCreator>(
        "SELECT p.id, p.title, p.description, p.creator_id, u.username as creator_username,
         p.created_at, p.expires_at
         FROM polls p
         JOIN users u ON p.creator_id = u.id
         WHERE p.expires_at <= datetime('now')
         ORDER BY p.created_at DESC",
    )
    .fetch_all(pool)
    .await
}

/// Retrieves a specific poll by its ID with creator information.
/// 
/// This function fetches a single poll from the database including
/// the creator's username for display purposes.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `poll_id` - Unique identifier of the poll to retrieve
/// 
/// # Returns
/// * `Ok(PollWithCreator)` - The poll with creator information
/// * `Err(sqlx::Error)` - Database error if poll not found or query fails
pub async fn get_poll_by_id(
    pool: &SqlitePool,
    poll_id: i64,
) -> Result<PollWithCreator, sqlx::Error> {
    sqlx::query_as::<_, PollWithCreator>(
        "SELECT p.id, p.title, p.description, p.creator_id, u.username as creator_username,
         p.created_at, p.expires_at
         FROM polls p
         JOIN users u ON p.creator_id = u.id
         WHERE p.id = ?",
    )
    .bind(poll_id)
    .fetch_one(pool)
    .await
}

/// Retrieves all voting options for a specific poll.
/// 
/// This function fetches all options for a poll including their
/// vote counts calculated from the votes table.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `poll_id` - ID of the poll to get options for
/// 
/// # Returns
/// * `Ok(Vec<PollOption>)` - Vector of poll options with vote counts
/// * `Err(sqlx::Error)` - Database error if query fails
pub async fn get_poll_options(
    pool: &SqlitePool,
    poll_id: i64,
) -> Result<Vec<PollOption>, sqlx::Error> {
    sqlx::query_as::<_, PollOption>(
        "SELECT o.id, o.poll_id, o.text, o.is_date, o.date_time,
         (SELECT COUNT(*) FROM votes v WHERE v.option_id = o.id) as vote_count
         FROM options o
         WHERE o.poll_id = ?
         ORDER BY o.id",
    )
    .bind(poll_id)
    .fetch_all(pool)
    .await
}

/// Retrieves all option IDs that a specific user has voted for in a poll.
/// 
/// This function is used to determine which options a user has already
/// voted for, enabling the UI to show their current voting status.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `poll_id` - ID of the poll to check votes for
/// * `user_id` - ID of the user whose votes to retrieve
/// 
/// # Returns
/// * `Ok(Vec<i64>)` - Vector of option IDs the user has voted for
/// * `Err(sqlx::Error)` - Database error if query fails
pub async fn get_user_votes(
    pool: &SqlitePool,
    poll_id: i64,
    user_id: i64,
) -> Result<Vec<i64>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT o.id
         FROM votes v
         JOIN options o ON v.option_id = o.id
         WHERE o.poll_id = ? AND v.user_id = ?",
    )
    .bind(poll_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|row| row.get::<i64, _>(0)).collect())
}

/// Creates a new poll with options in the database.
/// 
/// This function handles the complete poll creation process:
/// 1. Parses and validates the expiration date
/// 2. Creates the poll record in a transaction
/// 3. Parses comma-separated options
/// 4. Detects and handles date/time options
/// 5. Inserts all options for the poll
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `form` - New poll form data containing title, description, expiration, and options
/// * `user_id` - ID of the user creating the poll
/// 
/// # Returns
/// * `Ok(i64)` - The ID of the newly created poll
/// * `Err(sqlx::Error)` - Database error or invalid date format
/// 
/// # Date Format
/// Expiration dates should be in format: YYYY-MM-DDTHH:MM
/// Options can include dates in the same format for date-based voting
pub async fn create_poll(
    pool: &SqlitePool,
    form: &NewPollForm,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Parse expiration date
    let expires_at =
        match chrono::DateTime::parse_from_rfc3339(&format!("{}:00Z", &form.expires_at)) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => {
                error!("Invalid date format: {}", form.expires_at);
                return Err(sqlx::Error::ColumnDecode {
                    index: "".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid date format",
                    )),
                });
            }
        };

    // Insert poll
    let poll_id = sqlx::query(
        "INSERT INTO polls (title, description, creator_id, expires_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&form.title)
    .bind(&form.description)
    .bind(user_id)
    .bind(expires_at)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    // Parse and insert options
    let options = form
        .options
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>();

    for option in options {
        // Check if the option is a date/time
        let is_date = option.contains("T") && option.len() >= 16;
        let date_time = if is_date {
            match chrono::DateTime::parse_from_rfc3339(&format!("{}:00Z", option)) {
                Ok(dt) => Some(dt.with_timezone(&Utc)),
                Err(_) => None,
            }
        } else {
            None
        };

        sqlx::query("INSERT INTO options (poll_id, text, is_date, date_time) VALUES (?, ?, ?, ?)")
            .bind(poll_id)
            .bind(option)
            .bind(is_date)
            .bind(date_time)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    info!("New poll created with ID: {}", poll_id);
    Ok(poll_id)
}

/// Handles voting on a poll option (toggle functionality).
/// 
/// This function implements vote toggling - if the user has already
/// voted for the option, it removes their vote. If they haven't
/// voted for the option, it adds their vote.
/// 
/// # Vote Logic
/// - If user has already voted for this option: Remove the vote
/// - If user has not voted for this option: Add the vote
/// - Users can vote for multiple options in the same poll
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `option_id` - ID of the poll option to vote for/against
/// * `user_id` - ID of the user casting the vote
/// 
/// # Returns
/// * `Ok(())` - Vote operation completed successfully
/// * `Err(sqlx::Error)` - Database error if operation fails
pub async fn vote_on_poll(
    pool: &SqlitePool,
    option_id: i64,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    // Check if user has already voted for this option
    let existing_vote = sqlx::query("SELECT id FROM votes WHERE user_id = ? AND option_id = ?")
        .bind(user_id)
        .bind(option_id)
        .fetch_optional(pool)
        .await?;

    if existing_vote.is_some() {
        // User has already voted for this option, remove the vote
        sqlx::query("DELETE FROM votes WHERE user_id = ? AND option_id = ?")
            .bind(user_id)
            .bind(option_id)
            .execute(pool)
            .await?;

        info!("User {} removed vote for option {}", user_id, option_id);
    } else {
        // User has not voted for this option, add the vote
        sqlx::query("INSERT INTO votes (user_id, option_id) VALUES (?, ?)")
            .bind(user_id)
            .bind(option_id)
            .execute(pool)
            .await?;

        info!("User {} voted for option {}", user_id, option_id);
    }

    Ok(())
}

// Get poll results
// pub async fn get_poll_results(
//     pool: &SqlitePool,
//     poll_id: i64,
// ) -> Result<Vec<(PollOption, i64)>, sqlx::Error> {
//     let options = get_poll_options(pool, poll_id).await?;

//     let mut results = Vec::new();
//     for option in options {
//         let count = sqlx::query_scalar("SELECT COUNT(*) FROM votes WHERE option_id = ?")
//             .bind(option.id)
//             .fetch_one(pool)
//             .await?;

//         results.push((option, count));
//     }

//     Ok(results)
// }

/// Deletes a poll and all associated data (admin or creator only).
/// 
/// This function performs a cascading delete of a poll, removing:
/// 1. All votes for the poll's options
/// 2. All options for the poll
/// 3. The poll itself
/// 
/// # Permission Checks
/// - Admins can delete any poll
/// - Regular users can only delete polls they created
/// - Returns RowNotFound error if user lacks permission
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `poll_id` - ID of the poll to delete
/// * `user_id` - ID of the user requesting deletion
/// * `is_admin` - Whether the requesting user is an admin
/// 
/// # Returns
/// * `Ok(())` - Poll deleted successfully
/// * `Err(sqlx::Error)` - Database error or permission denied (RowNotFound)
pub async fn delete_poll(pool: &SqlitePool, poll_id: i64, user_id: i64, is_admin: bool) -> Result<(), sqlx::Error> {
    // First check if user has permission to delete this poll
    if !is_admin {
        let poll = sqlx::query_as::<_, crate::models::Poll>(
            "SELECT id, title, description, creator_id, created_at, expires_at FROM polls WHERE id = ?"
        )
        .bind(poll_id)
        .fetch_optional(pool)
        .await?;

        match poll {
            Some(poll) if poll.creator_id != user_id => {
                return Err(sqlx::Error::RowNotFound);
            }
            None => {
                return Err(sqlx::Error::RowNotFound);
            }
            _ => {} // User is the creator, proceed with deletion
        }
    }

    let mut tx = pool.begin().await?;

    // Delete all votes for this poll's options
    sqlx::query(
        "DELETE FROM votes WHERE option_id IN (SELECT id FROM options WHERE poll_id = ?)"
    )
    .bind(poll_id)
    .execute(&mut *tx)
    .await?;

    // Delete all options for this poll
    sqlx::query("DELETE FROM options WHERE poll_id = ?")
        .bind(poll_id)
        .execute(&mut *tx)
        .await?;

    // Delete the poll itself
    sqlx::query("DELETE FROM polls WHERE id = ?")
        .bind(poll_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    info!("Poll {} deleted by user {}", poll_id, user_id);
    Ok(())
}

/// Retrieves all users who voted for a specific poll option.
/// 
/// This function returns the list of users who cast votes for
/// a particular option, ordered by when they voted.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `option_id` - ID of the poll option to get voters for
/// 
/// # Returns
/// * `Ok(Vec<User>)` - Vector of users who voted for this option
/// * `Err(sqlx::Error)` - Database error if query fails
pub async fn get_voters_for_option(
    pool: &SqlitePool,
    option_id: i64,
) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT u.id, u.username, u.is_admin, u.created_at, u.password_hash
         FROM users u
         JOIN votes v ON u.id = v.user_id
         WHERE v.option_id = ?
         ORDER BY v.created_at ASC",
    )
    .bind(option_id)
    .fetch_all(pool)
    .await
}

/// Retrieves all voters for a poll with their complete voting choices.
/// 
/// This function returns each unique voter along with all the option IDs
/// they voted for in the specified poll. Used for detailed voter analysis.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `poll_id` - ID of the poll to get voters for
/// 
/// # Returns
/// * `Ok(Vec<(User, Vec<i64>)>)` - Vector of tuples containing each voter and their option IDs
/// * `Err(sqlx::Error)` - Database error if query fails
pub async fn get_poll_voters(
    pool: &SqlitePool,
    poll_id: i64,
) -> Result<Vec<(User, Vec<i64>)>, sqlx::Error> {
    // Get all users who voted in this poll
    let voters = sqlx::query_as::<_, User>(
        "SELECT DISTINCT u.id, u.username, u.is_admin, u.created_at, u.password_hash
         FROM users u
         JOIN votes v ON u.id = v.user_id
         JOIN options o ON v.option_id = o.id
         WHERE o.poll_id = ?
         ORDER BY u.username",
    )
    .bind(poll_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    
    for voter in voters {
        // Get all option IDs this user voted for in this poll
        let voted_options = sqlx::query_scalar::<_, i64>(
            "SELECT o.id
             FROM votes v
             JOIN options o ON v.option_id = o.id
             WHERE v.user_id = ? AND o.poll_id = ?
             ORDER BY o.id",
        )
        .bind(voter.id)
        .bind(poll_id)
        .fetch_all(pool)
        .await?;

        result.push((voter, voted_options));
    }

    Ok(result)
}

/// Retrieves comprehensive voting details for a poll.
/// 
/// This function aggregates all voting information for a poll into
/// a single structure containing the poll, all options with their voters,
/// and summary statistics.
/// 
/// # Data Collected
/// - Poll information with creator details
/// - All options with individual vote details and voter information
/// - Total vote count across all options
/// - Count of unique voters who participated
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `poll_id` - ID of the poll to get detailed information for
/// 
/// # Returns
/// * `Ok(PollVotingDetails)` - Complete voting details structure
/// * `Err(sqlx::Error)` - Database error if query fails
pub async fn get_poll_voting_details(
    pool: &SqlitePool,
    poll_id: i64,
) -> Result<PollVotingDetails, sqlx::Error> {
    // Get the poll
    let poll = get_poll_by_id(pool, poll_id).await?;
    
    // Get all options for this poll
    let options = get_poll_options(pool, poll_id).await?;
    
    let mut options_with_voters = Vec::new();
    let mut total_votes = 0;
    let mut all_voters = std::collections::HashSet::new();
    
    for option in options {
        // Get votes for this option with user information
        let votes_with_users = sqlx::query_as::<_, VoteWithUser>(
            "SELECT v.id as vote_id, v.user_id, u.username, v.option_id, v.created_at
             FROM votes v
             JOIN users u ON v.user_id = u.id
             WHERE v.option_id = ?
             ORDER BY v.created_at ASC",
        )
        .bind(option.id)
        .fetch_all(pool)
        .await?;
        
        total_votes += votes_with_users.len() as i64;
        
        // Track unique voters
        for vote in &votes_with_users {
            all_voters.insert(vote.user_id);
        }
        
        let option_with_voters = OptionWithVoters {
            id: option.id,
            poll_id: option.poll_id,
            text: option.text,
            is_date: option.is_date,
            date_time: option.date_time,
            vote_count: votes_with_users.len() as i64,
            voters: votes_with_users,
        };
        
        options_with_voters.push(option_with_voters);
    }
    
    Ok(PollVotingDetails {
        poll,
        options_with_voters,
        total_votes,
        total_voters: all_voters.len() as i64,
    })
}

/// Formats poll data into JSON structure for template rendering.
/// 
/// This function converts poll and voting data into a JSON structure
/// suitable for use in Tera templates, including vote counts, user voting
/// status, and expiration information.
/// 
/// # Template Data Included
/// - Poll basic information (title, description, creator, dates)
/// - Expiration status (is_expired boolean)
/// - All options with vote counts and user voting status
/// - Total vote count across all options
/// 
/// # Arguments
/// * `poll` - Poll information with creator details
/// * `options` - Array of poll options with vote counts
/// * `user_votes` - Array of option IDs the current user has voted for
/// 
/// # Returns
/// A JSON value containing all formatted poll data for template use
pub fn format_poll_for_template(
    poll: &PollWithCreator,
    options: &[PollOption],
    user_votes: &[i64],
) -> serde_json::Value {
    let options_json: Vec<serde_json::Value> = options
        .iter()
        .map(|option| {
            let is_voted = user_votes.contains(&option.id);

            serde_json::json!({
                "id": option.id,
                "text": option.text,
                "is_date": option.is_date,
                "date_time": option.date_time,
                "vote_count": option.vote_count,
                "is_voted": is_voted,
            })
        })
        .collect();

    let total_votes: i64 = options.iter().map(|o| o.vote_count).sum();

    serde_json::json!({
        "id": poll.id,
        "title": poll.title,
        "description": poll.description,
        "creator_id": poll.creator_id,
        "creator_username": poll.creator_username,
        "created_at": poll.created_at.to_rfc3339(),
        "expires_at": poll.expires_at.to_rfc3339(),
        "is_expired": poll.expires_at <= Utc::now(),
        "options": options_json,
        "total_votes": total_votes,
    })
}
