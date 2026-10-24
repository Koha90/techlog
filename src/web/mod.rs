//! HTTP interface for the TechLog application.

use std::path::PathBuf;

use axum::{
    Router,
    routing::{get, post},
};

mod forms;
mod handlers;
mod views;

/// Shared application configuration.
#[derive(Clone)]
struct AppState {
    database_path: PathBuf,
}

/// Creates the TechLog HTTP router.
pub fn router(database_path: PathBuf) -> Router {
    Router::new()
        .route("/", get(handlers::show_journal))
        .route(
            "/works",
            post(handlers::create_work),
        )
        .with_state(AppState { database_path })
}
