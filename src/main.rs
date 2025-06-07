#[macro_use]
extern crate rocket;
use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;
use rocket::fairing::AdHoc;
use dotenv::dotenv;
use std::env;

mod models;
mod controllers;
mod routes;
mod db;
mod auth;

use routes::*;
use db::init_db;

#[launch]
fn rocket() -> _ {
    // Load environment variables
    dotenv().ok();
    
    // Configure logging
    env_logger::init();
    
    rocket::build()
        .mount("/", routes![
            index,
            login_page,
            login_post,
            logout,
            dashboard,
            polls,
            poll_detail,
            create_poll_page,
            create_poll_post,
            vote_on_poll,
            admin_users,
            add_user_page,
            add_user_post,
            metrics
        ])
        .mount("/static", FileServer::from(relative!("src/static")))
        .attach(Template::fairing())
        .attach(AdHoc::try_on_ignite("Database Setup", init_db))
}
