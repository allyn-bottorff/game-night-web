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
use crate::models::poll::{NewPollForm, VoteForm};
use crate::models::user::{ChangePasswordForm, LoginForm, NewUserForm, ToggleRoleForm};

// Public routes

#[get("/")]
pub async fn index() -> Redirect {
    Redirect::to(uri!(login_page))
}

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

#[post("/login", data = "<form>")]
pub async fn login_post(
    form: Form<LoginForm>,
    cookies: &CookieJar<'_>,
    pool: &State<SqlitePool>,
) -> Result<Redirect, Flash<Redirect>> {
    crate::controllers::metrics::increment_login_attempt();

    let result = users::login_controller(pool, &form, cookies).await;

    match &result {
        Ok(_) => crate::controllers::metrics::increment_successful_login(),
        Err(_) => crate::controllers::metrics::increment_failed_login(),
    }

    result
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    users::logout_controller(cookies)
}

// Authenticated routes

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

#[post("/polls/<poll_id>/delete")]
pub async fn delete_poll(
    poll_id: i64,
    _admin: AdminUser,
    pool: &State<SqlitePool>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match polls::delete_poll(pool, poll_id).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!(dashboard)),
            "Poll deleted successfully.",
        )),
        Err(err) => Err(Flash::error(
            Redirect::to(uri!(poll_detail(poll_id))),
            format!("Failed to delete poll: {}", err),
        )),
    }
}

// User Profile routes

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

#[post("/profile/password", data = "<form>")]
pub async fn change_password(
    user: AuthenticatedUser,
    form: Form<ChangePasswordForm>,
    pool: &State<SqlitePool>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    users::change_password(pool, user.id, &form).await
}

// Admin routes

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

#[post("/admin/users/role", data = "<form>")]
pub async fn toggle_user_role(
    admin: AdminUser,
    form: Form<ToggleRoleForm>,
    pool: &State<SqlitePool>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    users::toggle_user_role(pool, form.user_id, form.set_admin, admin.id).await
}

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

#[post("/admin/users/add", data = "<form>")]
pub async fn add_user_post(
    _admin: AdminUser,
    form: Form<NewUserForm>,
    pool: &State<SqlitePool>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    users::add_user_controller(pool, &form).await
}

// Metrics endpoint
#[get("/metrics")]
pub async fn metrics_endpoint(pool: &State<SqlitePool>) -> String {
    crate::controllers::metrics::get_metrics(pool).await
}
