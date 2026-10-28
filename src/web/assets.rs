//! HTTP handler for embedded static assets.

use axum::{http::header, response::IntoResponse};

/// Serves the application stylesheet.
pub(super) async fn stylesheet() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "text/css; charset=utf-8",
        )],
        include_str!("../../assets/app.css"),
    )
}

/// Serves the application theme controller.
pub(super) async fn theme_script() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "text/javascript; charset=utf-8",
        )],
        include_str!("../../assets/theme.js"),
    )
}

/// Serves the employee selection controller.
pub(super) async fn performers_script() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "text/javascript; charset=utf-8",
        )],
        include_str!("../../assets/performers.js"),
    )
}
