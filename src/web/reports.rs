//! HTTP handlers for downloadable work reports.

use askama::Result;
use axum::{
    extract::{Query, State},
    http::header,
    response::IntoResponse,
};
use chrono::Local;

use crate::{reports::generate_work_report, storage::list_work_records_filtered};

use super::{
    AppState,
    filters::parse_journal_filter,
    forms::JournalQuery,
    handlers::{HandlerError, internal_error, open_connection},
};

/// Dowloads a DOCX report using the current journal filters.
///
/// Database access and document generation run outside
/// the asynchronous Tokio worker threads.
pub(super) async fn download_report(
    State(state): State<AppState>,
    Query(filters): Query<JournalQuery>,
) -> Result<impl IntoResponse, HandlerError> {
    let filter = parse_journal_filter(&filters)?;
    let database_path = state.database_path;
    let generated_on = Local::now().date_naive();

    let bytes = tokio::task::spawn_blocking(
        move || -> Result<Vec<u8>, String> {
            let conn = open_connection(&database_path).map_err(|error| error.to_string())?;

            let records =
                list_work_records_filtered(&conn, &filter).map_err(|error| error.to_string())?;

            generate_work_report(&records, &filter, generated_on).map_err(|error| error.to_string())
        },
    )
    .await
    .map_err(internal_error)?
    .map_err(internal_error)?;

    Ok((
        [
            (
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"techlog-report.docx\"",
            ),
        ],
        bytes,
    ))
}
