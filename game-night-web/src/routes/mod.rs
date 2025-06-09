//! # Routes Module
//!
//! This module contains all HTTP route handlers for the Game Night application.
//! Routes are organized by access level and functionality:
//!
//! ## Route Categories
//! - **Public Routes** - Login, logout, and landing pages
//! - **Authenticated Routes** - Dashboard, polls, voting, and profile management
//! - **Admin Routes** - User management and administrative functions
//! - **Utility Routes** - Metrics and monitoring endpoints
//!
//! ## Authentication
//! Routes use Rocket request guards to enforce authentication:
//! - `AuthenticatedUser` - Requires valid session
//! - `AdminUser` - Requires admin privileges
//!
//! ## Template Rendering
//! All routes use Tera templates with consistent context data including:
//! - Page title for SEO and navigation
//! - Current user information
//! - Flash messages for user feedback
//! - Page-specific data

use rocket::form::Form;
use rocket::get;
use rocket::http::{CookieJar, Status};
use rocket::post;
use rocket::response::{Flash, Redirect};
use rocket::uri;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use sqlx::SqlitePool;

use crate::auth::{AdminUser, AuthenticatedUser};
use crate::controllers::{polls, users};
use crate::models::{NewPollForm, VoteForm, ChangePasswordForm, LoginForm, NewUserForm, ToggleRoleForm};

// ============================================================================
// Public routes (no authentication required)
// ============================================================================

/// Root route that redirects to the login page.
/// 
/// This is the main entry point for the application when users
/// visit the root URL without being authenticated.
#[get("/")]
pub async fn index() -> Redirect {
    Redirect::to(uri!(login_page))
}

/// Displays the user login page.
/// 
/// This route renders the login form template with any flash messages
/// from previous login attempts or redirects.
/// 
/// # Parameters
/// * `flash` - Optional flash message from previous requests
/// 
/// # Returns
/// Login page template with flash messages if present
#[get("/login")]
pub async fn login_page(flash: Option<rocket::request::FlashMessage<'_>>) -> Template {
    Template::render(
        "login",
        context! {
            title: "Login - Platform Engineering Game Night",
            flash: flash.map(|msg| (msg.kind().to_string(), msg.message().to_string())),
        },
    )
}

/// Handles user login form submission.
/// 
/// This route processes login credentials, updates metrics,
/// and either redirects to the dashboard on success or back
/// to the login page with an error message.
/// 
/// # Parameters
/// * `form` - Login form data (username and password)
/// * `cookies` - Cookie jar for setting session cookies
/// * `pool` - Database connection pool
/// 
/// # Returns
/// * `Ok(Redirect)` - Redirects to dashboard on successful login
/// * `Err(Flash<Redirect>)` - Redirects to login page with error
#[post("/login", data = "<form>")]
pub async fn login_post(
    form: Form<LoginForm>,
    cookies: &CookieJar<'_>,
    pool: &State<SqlitePool>,
) -> Result<Redirect, Flash<Redirect>> {
    crate::db::increment_login_attempt();

    let result = users::login_controller(pool, &form, cookies).await;

    match &result {
        Ok(_) => crate::db::increment_successful_login(),
        Err(_) => crate::db::increment_failed_login(),
    }

    result
}

/// Handles user logout by clearing session cookies.
/// 
/// This route logs out the current user and redirects to the
/// login page with a confirmation message.
/// 
/// # Parameters
/// * `cookies` - Cookie jar for clearing session cookies
/// 
/// # Returns
/// Flash redirect to login page with logout confirmation
#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    users::logout_controller(cookies)
}

// ============================================================================
// Authenticated routes (require valid session)
// ============================================================================

/// Main dashboard page showing active and expired polls.
/// 
/// This is the primary landing page for authenticated users,
/// displaying an overview of all polls in the system.
/// 
/// # Parameters
/// * `user` - Authenticated user (enforced by request guard)
/// * `pool` - Database connection pool
/// * `flash` - Optional flash messages from previous actions
/// 
/// # Returns
/// * `Ok(Template)` - Dashboard template with poll data
/// * `Err(Status)` - Internal server error if database query fails
#[get("/dashboard")]
pub async fn dashboard(
    user: AuthenticatedUser,
    pool: &State<SqlitePool>,
    flash: Option<rocket::request::FlashMessage<'_>>,
) -> Result<Template, Status> {
    let active_polls = polls::get_active_polls(pool)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let expired_polls = polls::get_expired_polls(pool)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Template::render(
        "dashboard",
        context! {
            title: "Dashboard - Platform Engineering Game Night",
            user: user.user,
            active_polls: active_polls,
            expired_polls: expired_polls,
            flash: flash.map(|msg| (msg.kind().to_string(), msg.message().to_string())),
        },
    ))
}

/// Displays all polls page with active and expired polls.
/// 
/// This route provides a comprehensive view of all polls in the system,
/// similar to the dashboard but focused specifically on poll listing.
/// 
/// # Parameters
/// * `user` - Authenticated user (enforced by request guard)
/// * `pool` - Database connection pool
/// 
/// # Returns
/// * `Ok(Template)` - Polls page template with poll data
/// * `Err(Status)` - Internal server error if database query fails
#[get("/polls")]
pub async fn get_polls(
    user: AuthenticatedUser,
    pool: &State<SqlitePool>,
) -> Result<Template, Status> {
    let active_polls = polls::get_active_polls(pool)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let expired_polls = polls::get_expired_polls(pool)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Template::render(
        "polls",
        context! {
            title: "All Polls - Platform Engineering Game Night",
            user: user.user,
            active_polls: active_polls,
            expired_polls: expired_polls,
        },
    ))
}

/// Displays detailed view of a specific poll with voting options.
/// 
/// This route shows a poll's details, options, vote counts, and allows
/// users to cast or change their votes if the poll is still active.
/// 
/// # Parameters
/// * `poll_id` - Unique identifier of the poll to display
/// * `user` - Authenticated user (enforced by request guard)
/// * `pool` - Database connection pool
/// * `flash` - Optional flash messages from voting or other actions
/// 
/// # Returns
/// * `Ok(Template)` - Poll detail template with voting interface
/// * `Err(Status::NotFound)` - If poll doesn't exist
/// * `Err(Status::InternalServerError)` - If database query fails
#[get("/polls/<poll_id>")]
pub async fn poll_detail(
    poll_id: i64,
    user: AuthenticatedUser,
    pool: &State<SqlitePool>,
    flash: Option<rocket::request::FlashMessage<'_>>,
) -> Result<Template, Status> {
    let poll = polls::get_poll_by_id(pool, poll_id)
        .await
        .map_err(|_| Status::NotFound)?;

    let options = polls::get_poll_options(pool, poll_id)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let user_votes = polls::get_user_votes(pool, poll_id, user.id)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let poll_data = polls::format_poll_for_template(&poll, &options, &user_votes);

    Ok(Template::render(
        "poll_detail",
        context! {
            title: format!("{} - Platform Engineering Game Night", poll.title),
            user: user.user,
            poll: poll_data,
            flash: flash.map(|msg| (msg.kind().to_string(), msg.message().to_string())),
        },
    ))
}

/// Displays detailed voter information for a poll (creator/admin only).
/// 
/// This route shows who voted for each option in a poll. Access is restricted
/// to the poll creator and admin users for privacy reasons.
/// 
/// # Access Control
/// - Poll creators can view voters for their own polls
/// - Admin users can view voters for any poll
/// - Regular users cannot access this information
/// 
/// # Parameters
/// * `poll_id` - Unique identifier of the poll
/// * `user` - Authenticated user (enforced by request guard)
/// * `pool` - Database connection pool
/// * `flash` - Optional flash messages
/// 
/// # Returns
/// * `Ok(Template)` - Voters page with detailed voting information
/// * `Err(Status::NotFound)` - If poll doesn't exist
/// * `Err(Status::Forbidden)` - If user lacks permission
/// * `Err(Status::InternalServerError)` - If database query fails
#[get("/polls/<poll_id>/voters")]
pub async fn poll_voters(
    poll_id: i64,
    user: AuthenticatedUser,
    pool: &State<SqlitePool>,
    flash: Option<rocket::request::FlashMessage<'_>>,
) -> Result<Template, Status> {
    // Get poll to check permissions
    let poll = polls::get_poll_by_id(pool, poll_id)
        .await
        .map_err(|_| Status::NotFound)?;

    // Only allow poll creator or admins to see who voted
    if !user.is_admin && poll.creator_id != user.id {
        return Err(Status::Forbidden);
    }

    let voting_details = polls::get_poll_voting_details(pool, poll_id)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Template::render(
        "poll_voters",
        context! {
            title: format!("Voters for {} - Platform Engineering Game Night", poll.title),
            user: user.user,
            voting_details: voting_details,
            flash: flash.map(|msg| (msg.kind().to_string(), msg.message().to_string())),
        },
    ))
}

/// Displays the poll creation form page.
/// 
/// This route renders the form for creating new polls, including
/// fields for title, description, expiration date, and options.
/// 
/// # Parameters
/// * `user` - Authenticated user (enforced by request guard)
/// * `flash` - Optional flash messages from previous creation attempts
/// 
/// # Returns
/// Poll creation form template
#[get("/polls/create")]
pub async fn create_poll_page(
    user: AuthenticatedUser,
    flash: Option<rocket::request::FlashMessage<'_>>,
) -> Template {
    Template::render(
        "create_poll",
        context! {
            title: "Create Poll - Platform Engineering Game Night",
            user: user.user,
            flash: flash.map(|msg| (msg.kind().to_string(), msg.message().to_string())),
        },
    )
}

/// Handles poll creation form submission.
/// 
/// This route processes the new poll form data, creates the poll
/// and its options in the database, and redirects to the new poll's
/// detail page on success.
/// 
/// # Parameters
/// * `user` - Authenticated user (enforced by request guard)
/// * `form` - New poll form data
/// * `pool` - Database connection pool
/// 
/// # Returns
/// * `Ok(Redirect)` - Redirects to new poll detail page on success
/// * `Err(Flash<Redirect>)` - Redirects to creation page with error
#[post("/polls/create", data = "<form>")]
pub async fn create_poll_post(
    user: AuthenticatedUser,
    form: Form<NewPollForm>,
    pool: &State<SqlitePool>,
) -> Result<Redirect, Flash<Redirect>> {
    match polls::create_poll(pool, &form, user.id).await {
        Ok(poll_id) => Ok(Redirect::to(uri!(poll_detail(poll_id)))),
        Err(err) => Err(Flash::error(
            Redirect::to(uri!(create_poll_page)),
            format!("Failed to create poll: {}", err),
        )),
    }
}

/// Handles voting on poll options (toggle functionality).
/// 
/// This route processes vote submissions with the following logic:
/// - If user already voted for the option: remove their vote
/// - If user hasn't voted for the option: add their vote
/// - Prevents voting on expired polls
/// 
/// # Parameters
/// * `poll_id` - Unique identifier of the poll
/// * `user` - Authenticated user (enforced by request guard)
/// * `form` - Vote form data containing option ID
/// * `pool` - Database connection pool
/// 
/// # Returns
/// * `Ok(Redirect)` - Redirects back to poll detail page
/// * `Err(Flash<Redirect>)` - Redirects with error message
#[post("/polls/<poll_id>/vote", data = "<form>")]
pub async fn vote_on_poll(
    poll_id: i64,
    user: AuthenticatedUser,
    form: Form<VoteForm>,
    pool: &State<SqlitePool>,
) -> Result<Redirect, Flash<Redirect>> {
    // Check if poll is active
    let poll = match polls::get_poll_by_id(pool, poll_id).await {
        Ok(poll) => poll,
        Err(_) => {
            return Err(Flash::error(
                Redirect::to(uri!(poll_detail(poll_id))),
                "Poll not found.",
            ));
        }
    };

    if poll.expires_at <= chrono::Utc::now() {
        return Err(Flash::error(
            Redirect::to(uri!(poll_detail(poll_id))),
            "Cannot vote on expired poll.",
        ));
    }

    match polls::vote_on_poll(pool, form.option_id, user.id).await {
        Ok(_) => Ok(Redirect::to(uri!(poll_detail(poll_id)))),
        Err(err) => Err(Flash::error(
            Redirect::to(uri!(poll_detail(poll_id))),
            format!("Failed to cast vote: {}", err),
        )),
    }
}

/// Handles poll deletion (creator/admin only).
/// 
/// This route deletes a poll and all associated data including
/// options and votes. Access is restricted to the poll creator
/// and admin users.
/// 
/// # Access Control
/// - Poll creators can delete their own polls
/// - Admin users can delete any poll
/// - Regular users cannot delete others' polls
/// 
/// # Parameters
/// * `poll_id` - Unique identifier of the poll to delete
/// * `user` - Authenticated user (enforced by request guard)
/// * `pool` - Database connection pool
/// 
/// # Returns
/// * `Ok(Flash<Redirect>)` - Success redirect to dashboard
/// * `Err(Flash<Redirect>)` - Error redirect with message
#[post("/polls/<poll_id>/delete")]
pub async fn delete_poll(
    poll_id: i64,
    user: AuthenticatedUser,
    pool: &State<SqlitePool>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match polls::delete_poll(pool, poll_id, user.id, user.is_admin).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!(dashboard)),
            "Poll deleted successfully.",
        )),
        Err(sqlx::Error::RowNotFound) => Err(Flash::error(
            Redirect::to(uri!(poll_detail(poll_id))),
            "You don't have permission to delete this poll.",
        )),
        Err(err) => Err(Flash::error(
            Redirect::to(uri!(poll_detail(poll_id))),
            format!("Failed to delete poll: {}", err),
        )),
    }
}

// ============================================================================
// User Profile routes
// ============================================================================

/// Displays the user profile page with statistics.
/// 
/// This route shows the user's profile information including
/// statistics about polls created and votes cast.
/// 
/// # Parameters
/// * `user` - Authenticated user (enforced by request guard)
/// * `pool` - Database connection pool
/// * `flash` - Optional flash messages from profile updates
/// 
/// # Returns
/// * `Ok(Template)` - Profile page template with user statistics
/// * `Err(Status::InternalServerError)` - If database query fails
#[get("/profile")]
pub async fn profile(
    user: AuthenticatedUser,
    pool: &State<SqlitePool>,
    flash: Option<rocket::request::FlashMessage<'_>>,
) -> Result<Template, Status> {
    // Get user statistics
    let (polls_created, votes_cast) = users::get_user_stats(pool, user.id)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Template::render(
        "profile",
        context! {
            title: "User Profile - Platform Engineering Game Night",
            user: user.user,
            polls_created: polls_created,
            votes_cast: votes_cast,
            flash: flash.map(|msg| (msg.kind().to_string(), msg.message().to_string())),
        },
    ))
}

/// Handles password change requests.
/// 
/// This route processes password change forms, validates the current
/// password, and updates the user's password hash in the database.
/// 
/// # Parameters
/// * `user` - Authenticated user (enforced by request guard)
/// * `form` - Password change form data
/// * `pool` - Database connection pool
/// 
/// # Returns
/// * `Ok(Flash<Redirect>)` - Success redirect to profile page
/// * `Err(Flash<Redirect>)` - Error redirect to profile page
#[post("/profile/password", data = "<form>")]
pub async fn change_password(
    user: AuthenticatedUser,
    form: Form<ChangePasswordForm>,
    pool: &State<SqlitePool>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    users::change_password(pool, user.id, &form).await
}

// ============================================================================
// Admin routes (require admin privileges)
// ============================================================================

/// Displays the admin user management page.
/// 
/// This route shows all users in the system and provides admin
/// controls for managing user roles and accounts.
/// 
/// # Access Control
/// Requires admin privileges (enforced by AdminUser request guard)
/// 
/// # Parameters
/// * `admin` - Admin user (enforced by request guard)
/// * `pool` - Database connection pool
/// * `flash` - Optional flash messages from admin actions
/// 
/// # Returns
/// * `Ok(Template)` - Admin users page template
/// * `Err(Status::InternalServerError)` - If database query fails
#[get("/admin/users")]
pub async fn admin_users(
    admin: AdminUser,
    pool: &State<SqlitePool>,
    flash: Option<rocket::request::FlashMessage<'_>>,
) -> Result<Template, Status> {
    let users = users::get_all_users(pool)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Template::render(
        "admin_users",
        context! {
            title: "Manage Users - Platform Engineering Game Night",
            user: admin.user,
            users: users,
            flash: flash.map(|msg| (msg.kind().to_string(), msg.message().to_string())),
        },
    ))
}

/// Handles user role changes (promote/demote admin status).
/// 
/// This route allows admins to change user roles between regular
/// user and admin status. Includes safety checks to prevent
/// admins from demoting themselves.
/// 
/// # Access Control
/// Requires admin privileges (enforced by AdminUser request guard)
/// 
/// # Parameters
/// * `admin` - Admin user performing the action
/// * `form` - Role toggle form data
/// * `pool` - Database connection pool
/// 
/// # Returns
/// * `Ok(Flash<Redirect>)` - Success redirect to admin users page
/// * `Err(Flash<Redirect>)` - Error redirect with message
#[post("/admin/users/role", data = "<form>")]
pub async fn toggle_user_role(
    admin: AdminUser,
    form: Form<ToggleRoleForm>,
    pool: &State<SqlitePool>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    users::toggle_user_role(pool, form.user_id, form.set_admin, admin.id).await
}

/// Displays the add user form page (admin only).
/// 
/// This route renders the form for creating new user accounts,
/// including options for setting admin privileges.
/// 
/// # Access Control
/// Requires admin privileges (enforced by AdminUser request guard)
/// 
/// # Parameters
/// * `admin` - Admin user (enforced by request guard)
/// * `flash` - Optional flash messages from previous creation attempts
/// 
/// # Returns
/// Add user form template
#[get("/admin/users/add")]
pub async fn add_user_page(
    admin: AdminUser,
    flash: Option<rocket::request::FlashMessage<'_>>,
) -> Template {
    Template::render(
        "add_user",
        context! {
            title: "Add User - Platform Engineering Game Night",
            user: admin.user,
            flash: flash.map(|msg| (msg.kind().to_string(), msg.message().to_string())),
        },
    )
}

/// Handles new user creation form submission (admin only).
/// 
/// This route processes new user forms, validates the data,
/// and creates new user accounts in the database.
/// 
/// # Access Control
/// Requires admin privileges (enforced by AdminUser request guard)
/// 
/// # Parameters
/// * `_admin` - Admin user (authentication only, not used in logic)
/// * `form` - New user form data
/// * `pool` - Database connection pool
/// 
/// # Returns
/// * `Ok(Flash<Redirect>)` - Success redirect to admin users page
/// * `Err(Flash<Redirect>)` - Error redirect to add user page
#[post("/admin/users/add", data = "<form>")]
pub async fn add_user_post(
    _admin: AdminUser,
    form: Form<NewUserForm>,
    pool: &State<SqlitePool>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    users::add_user_controller(pool, &form).await
}

// ============================================================================
// Utility routes (monitoring and metrics)
// ============================================================================

/// Prometheus metrics endpoint for monitoring and observability.
/// 
/// This route exposes application metrics in Prometheus format for
/// scraping by monitoring systems. Metrics include database statistics,
/// login attempts, and other operational data.
/// 
/// # Public Access
/// This endpoint is intentionally public to allow monitoring systems
/// to scrape metrics without authentication.
/// 
/// # Parameters
/// * `pool` - Database connection pool for updating metrics
/// 
/// # Returns
/// Plain text response in Prometheus exposition format
#[get("/metrics")]
pub async fn metrics_endpoint(pool: &State<SqlitePool>) -> String {
    crate::db::get_metrics(pool).await
}
