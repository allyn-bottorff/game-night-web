extern crate rocket;
use dotenv::dotenv;
use rocket::fairing::AdHoc;
use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;
use std::env;

mod auth;
mod controllers;
mod db;
mod models;
mod routes;

use routes::*;

#[rocket::launch]
fn rocket() -> _ {
    // Load environment variables
    dotenv().ok();

    // Configure logging
    env_logger::init();

    rocket::build()
        .mount(
            "/",
            rocket::routes![
                index,
                login_page,
                login_post,
                logout,
                dashboard,
                get_polls,
                poll_detail,
                create_poll_page,
                create_poll_post,
                vote_on_poll,
                profile,
                change_password,
                admin_users,
                add_user_page,
                add_user_post,
                toggle_user_role,
                metrics_endpoint
            ],
        )
        .mount("/static", FileServer::from(relative!("src/static")))
        .attach(Template::fairing())
        .attach(AdHoc::try_on_ignite("Database Setup", |rocket| async {
            let pool = db::init_pool().await;

            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("failed to run database migrations");
            Ok(rocket.manage(pool))
        }))
}
