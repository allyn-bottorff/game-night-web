use axum::{
    Extension, Router,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{
    ConnectOptions, Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{env, net::SocketAddr, sync::Arc};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod db;
mod handlers;
mod metrics;
mod models;

use auth::auth_middleware;
use db::init_db;
use metrics::*;

#[derive(Clone)]
pub struct AppState {
    db: Pool<Sqlite>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize environment variables from .env file if it exists
    dotenv().ok();

    // Initialize the logger
    // env_logger::init();

    // Initialize the tracing subscriber
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Database setup
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:game_night.db".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await?;

    // Initialize the database schema
    init_db(&pool).await?;

    let app_state = AppState { db: pool };

    // Initialize metrics
    init_metrics();

    // // Configure CORS
    // let cors = CorsLayer::new()
    //     .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    //     .allow_headers(Any)
    //     .allow_origin(Any);

    // Define routes
    let app = Router::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .route("/api/health", get(health_check))
        .route("/api/metrics", get(metrics_handler))
        .route("/api/login", post(handlers::auth::login))
        .route("/api/register", post(handlers::auth::register))
        // User management
        .route("/api/users", get(handlers::users::list_all_users))
        .route("/api/users", post(handlers::users::create_new_user))
        // Poll management
        .route("/api/polls", get(handlers::polls::get_all_polls))
        .route("/api/polls", post(handlers::polls::create_new_poll))
        .route("/api/polls/{id}", get(handlers::polls::get_all_polls))
        .route("/api/polls/{id}", put(handlers::polls::update_poll))
        .route("/api/polls/{id}/vote", post(handlers::polls::add_vote))
        .route("/api/polls/{id}/results", get(handlers::polls::get_results))
        // .layer(cors)
        .with_state(app_state.clone())
        .layer(CorsLayer::very_permissive());
    // Apply auth middleware to protected routes
    // .route_layer(axum::middleware::from_fn_with_state(
    //     app_state.clone(),
    //     auth_middleware,
    // ));

    // Define address to listen on
    let addr = env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse::<SocketAddr>()?;

    tracing::info!("Starting server on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app).await?;

    Ok(())
}

// Health check endpoint
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

// Metrics endpoint
async fn metrics_handler() -> impl IntoResponse {
    let metrics = get_metrics();
    (StatusCode::OK, metrics)
}
