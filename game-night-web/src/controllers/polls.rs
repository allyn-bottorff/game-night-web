use chrono::Utc;
use log::{error, info};
use sqlx::{Row, SqlitePool};

use crate::models::poll::{NewPollForm, PollOption, PollWithCreator};

// Get active polls
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

// Get expired polls
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

// Get a single poll by ID
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

// Get poll options for a poll
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

// Get user votes for a poll
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

// Create a new poll
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

// Vote on a poll
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

// Delete a poll (admin only)
pub async fn delete_poll(pool: &SqlitePool, poll_id: i64) -> Result<(), sqlx::Error> {
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

    info!("Poll {} deleted", poll_id);
    Ok(())
}

// Format poll data for template
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
        "creator_username": poll.creator_username,
        "created_at": poll.created_at.to_rfc3339(),
        "expires_at": poll.expires_at.to_rfc3339(),
        "is_expired": poll.expires_at <= Utc::now(),
        "options": options_json,
        "total_votes": total_votes,
    })
}
