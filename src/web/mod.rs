//! HTTP interface for the TechLog application.

use std::path::PathBuf;

use axum::{
    Router,
    routing::{get, post},
};

mod assets;
mod filters;
mod forms;
mod handlers;
mod reports;
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
        .route(
            "/assets/app.css",
            get(assets::stylesheet),
        )
        .route(
            "/assets/theme.js",
            get(assets::theme_script),
        )
        .route(
            "/reports/docx",
            get(reports::download_report),
        )
        .with_state(AppState { database_path })
}
