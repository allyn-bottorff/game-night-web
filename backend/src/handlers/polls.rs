// handlers/polls.rs
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    AppState,
    db::{
        add_poll_option, create_poll, get_poll, get_poll_options, get_user_vote, get_vote_counts,
        list_polls, update_poll_statuses, vote,
    },
    models::{
        CreatePollOptionRequest, CreatePollRequest, ErrorResponse, Poll, PollOption,
        PollOptionWithVotes, PollResponse, PollResult, UserInfo, VoteRequest,
    },
};

// List all polls
pub async fn get_all_polls(
    State(state): State<AppState>,
    Extension(current_user): Extension<UserInfo>,
) -> Response {
    // Update poll statuses based on expiration dates
    if let Err(_) = update_poll_statuses(&state.db).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to update poll statuses".to_string(),
            }),
        )
            .into_response();
    }

    // Get active polls by default
    match list_polls(&state.db, true).await {
        Ok(polls) => (StatusCode::OK, Json(polls)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to retrieve polls".to_string(),
            }),
        )
            .into_response(),
    }
}

// Create a new poll
pub async fn create_new_poll(
    State(state): State<AppState>,
    Extension(current_user): Extension<UserInfo>,
    Json(payload): Json<CreatePollRequest>,
) -> Response {
    // Validate input
    if payload.title.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Poll title cannot be empty".to_string(),
            }),
        )
            .into_response();
    }

    if payload.options.len() < 2 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Poll must have at least 2 options".to_string(),
            }),
        )
            .into_response();
    }

    // Create the poll
    let poll_id = match create_poll(
        &state.db,
        &payload.title,
        payload.description.as_deref(),
        current_user.id,
        payload.expires_at,
    )
    .await
    {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create poll".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Add poll options
    for option in payload.options {
        if let Err(_) =
            add_poll_option(&state.db, poll_id, &option.text, option.datetime_option).await
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to add poll options".to_string(),
                }),
            )
                .into_response();
        }
    }

    // Get the created poll with its options
    match get_poll(&state.db, poll_id).await {
        Ok(Some(poll)) => match get_poll_options(&state.db, poll_id).await {
            Ok(options) => {
                let response = PollResponse {
                    poll,
                    options,
                    user_vote: None, // New poll, no votes yet
                };
                (StatusCode::CREATED, Json(response)).into_response()
            }
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to retrieve poll options".to_string(),
                }),
            )
                .into_response(),
        },
        Ok(None) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Poll was created but could not be retrieved".to_string(),
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to retrieve created poll".to_string(),
            }),
        )
            .into_response(),
    }
}

// Get a specific poll by ID
pub async fn get_single_poll(
    State(state): State<AppState>,
    Extension(current_user): Extension<UserInfo>,
    Path(poll_id): Path<i64>,
) -> Response {
    // Update poll statuses based on expiration dates
    if let Err(_) = update_poll_statuses(&state.db).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to update poll statuses".to_string(),
            }),
        )
            .into_response();
    }

    // Get the poll
    match get_poll(&state.db, poll_id).await {
        Ok(Some(poll)) => {
            // Get the poll options
            match get_poll_options(&state.db, poll_id).await {
                Ok(options) => {
                    // Get the user's vote for this poll, if any
                    let user_vote = match get_user_vote(&state.db, poll_id, current_user.id).await {
                        Ok(vote) => vote,
                        Err(_) => None,
                    };

                    let response = PollResponse {
                        poll,
                        options,
                        user_vote,
                    };
                    (StatusCode::OK, Json(response)).into_response()
                }
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to retrieve poll options".to_string(),
                    }),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Poll not found".to_string(),
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to retrieve poll".to_string(),
            }),
        )
            .into_response(),
    }
}

// Update a poll
pub async fn update_poll(
    State(state): State<AppState>,
    Extension(current_user): Extension<UserInfo>,
    Path(poll_id): Path<i64>,
    Json(payload): Json<UpdatePollRequest>,
) -> Response {
    // Get the poll to check if the user is the owner
    match get_poll(&state.db, poll_id).await {
        Ok(Some(poll)) => {
            // Only the poll creator or an admin can update the poll
            if poll.created_by != current_user.id && !current_user.is_admin {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        error: "You don't have permission to update this poll".to_string(),
                    }),
                )
                    .into_response();
            }

            // Update the poll in the database
            match sqlx::query(
                r#"
                UPDATE polls
                SET title = ?, description = ?, expires_at = ?, is_active = ?
                WHERE id = ?
                "#,
            )
            .bind(&payload.title)
            .bind(&payload.description)
            .bind(payload.expires_at)
            .bind(payload.is_active)
            .bind(poll_id)
            .execute(&state.db)
            .await
            {
                Ok(_) => (StatusCode::OK, Json(())).into_response(),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to update poll".to_string(),
                    }),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Poll not found".to_string(),
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to retrieve poll".to_string(),
            }),
        )
            .into_response(),
    }
}

// Vote on a poll
pub async fn add_vote(
    State(state): State<AppState>,
    Extension(current_user): Extension<UserInfo>,
    Path(poll_id): Path<i64>,
    Json(payload): Json<VoteRequest>,
) -> Response {
    // Check if the poll exists and is active
    match get_poll(&state.db, poll_id).await {
        Ok(Some(poll)) => {
            // Check if the poll is active
            if !poll.is_active {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "This poll is no longer active".to_string(),
                    }),
                )
                    .into_response();
            }

            // Check if the poll has expired
            if let Some(expires_at) = poll.expires_at {
                if expires_at < Utc::now() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "This poll has expired".to_string(),
                        }),
                    )
                        .into_response();
                }
            }

            // Check if the option exists for this poll
            let option_exists = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM poll_options WHERE id = ? AND poll_id = ?",
            )
            .bind(payload.option_id)
            .bind(poll_id)
            .fetch_one(&state.db)
            .await;

            match option_exists {
                Ok(count) if count > 0 => {
                    // Submit the vote
                    match vote(&state.db, poll_id, payload.option_id, current_user.id).await {
                        Ok(_) => (StatusCode::OK, Json(())).into_response(),
                        Err(_) => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Failed to submit vote".to_string(),
                            }),
                        )
                            .into_response(),
                    }
                }
                Ok(_) => (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Invalid poll option".to_string(),
                    }),
                )
                    .into_response(),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to validate poll option".to_string(),
                    }),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Poll not found".to_string(),
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to retrieve poll".to_string(),
            }),
        )
            .into_response(),
    }
}

// Get poll results
pub async fn get_results(
    State(state): State<AppState>,
    Extension(current_user): Extension<UserInfo>,
    Path(poll_id): Path<i64>,
) -> Response {
    // Get the poll
    match get_poll(&state.db, poll_id).await {
        Ok(Some(poll)) => {
            // Get the poll options
            match get_poll_options(&state.db, poll_id).await {
                Ok(options) => {
                    // Get vote counts for each option
                    match get_vote_counts(&state.db, poll_id).await {
                        Ok(vote_counts) => {
                            // Convert to a map for easier lookup
                            let mut vote_count_map = std::collections::HashMap::new();
                            for (option_id, count) in vote_counts {
                                vote_count_map.insert(option_id, count);
                            }

                            // Calculate total votes
                            let total_votes: i64 = vote_count_map.values().sum();

                            // Build the poll options with vote counts
                            let options_with_votes: Vec<PollOptionWithVotes> = options
                                .into_iter()
                                .map(|option| {
                                    let vote_count = *vote_count_map.get(&option.id).unwrap_or(&0);
                                    let percentage = if total_votes > 0 {
                                        (vote_count as f64 / total_votes as f64) * 100.0
                                    } else {
                                        0.0
                                    };

                                    PollOptionWithVotes {
                                        id: option.id,
                                        text: option.text,
                                        datetime_option: option.datetime_option,
                                        vote_count,
                                        percentage,
                                    }
                                })
                                .collect();

                            // Get the user's vote for this poll, if any
                            let user_vote =
                                match get_user_vote(&state.db, poll_id, current_user.id).await {
                                    Ok(vote) => vote,
                                    Err(_) => None,
                                };

                            let result = PollResult {
                                poll,
                                options: options_with_votes,
                                total_votes,
                                user_vote,
                            };

                            (StatusCode::OK, Json(result)).into_response()
                        }
                        Err(_) => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Failed to retrieve vote counts".to_string(),
                            }),
                        )
                            .into_response(),
                    }
                }
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to retrieve poll options".to_string(),
                    }),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Poll not found".to_string(),
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to retrieve poll".to_string(),
            }),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdatePollRequest {
    pub title: String,
    pub description: Option<String>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub is_active: bool,
}
