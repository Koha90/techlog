//! HTTP request handlers for the work journal.

use std::{path::Path, time::Duration};

use askama::Template;
use axum::{
    Form,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Redirect},
};
use chrono::{Local, NaiveDate};
use rusqlite::Connection;

use crate::{
    domain::work_record::WorkRecordDraft,
    storage::{WorkRecordFilter, list_work_records_filtered, save_work_record},
};

use super::{
    AppState,
    forms::{JournalQuery, NewWorkForm},
    views::{JournalTemplate, RecordView},
};

type HandlerError = (StatusCode, String);

/// Converts an internal error into an HTTP 500 response.
fn internal_error(error: impl std::fmt::Display) -> HandlerError {
    eprintln!("TechLog error: {error}");

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Внутренняя ошибка сервера".into(),
    )
}

/// Opens a SQLite connection with foreign key enforcement.
fn open_connection(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;

    conn.busy_timeout(Duration::from_secs(5))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    Ok(conn)
}

/// Parses an optional date from a journal Query parameter.
fn parse_filter_date(value: &str) -> Result<Option<NaiveDate>, HandlerError> {
    let value = value.trim();

    if value.is_empty() {
        return Ok(None);
    }

    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Некорректная дата в фильтре".into(),
            )
        })
}

/// Normalizes an optional text filter.
fn text_filter(value: &str) -> Option<String> {
    let value = value.trim();

    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

/// Display work records matching the query parameters.
pub(super) async fn show_journal(
    State(state): State<AppState>,
    Query(filters): Query<JournalQuery>,
) -> Result<Html<String>, HandlerError> {
    let filter = WorkRecordFilter {
        from: parse_filter_date(&filters.from)?,
        to: parse_filter_date(&filters.to)?,
        department: text_filter(&filters.department),
        workshop: text_filter(&filters.workshop),
        performer: text_filter(&filters.performer),
    };

    if let (Some(form), Some(to)) = (filter.from, filter.to)
        && form > to
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Начальная дата позже конечной".into(),
        ));
    }

    let database_path = state.database_path;

    let works = tokio::task::spawn_blocking(move || {
        let conn = open_connection(&database_path)?;
        list_work_records_filtered(&conn, &filter)
    })
    .await
    .map_err(internal_error)?
    .map_err(internal_error)?;

    let records: Vec<RecordView> = works
        .into_iter()
        .map(RecordView::from)
        .collect();

    let page = JournalTemplate {
        today: Local::now()
            .date_naive()
            .to_string(),
        has_records: !records.is_empty(),
        records,
        filters,
    };

    let html = page
        .render()
        .map_err(internal_error)?;

    Ok(Html(html))
}

/// Validates and saves a submitted technical work record.
pub(super) async fn create_work(
    State(state): State<AppState>,
    Form(form): Form<NewWorkForm>,
) -> Result<Redirect, HandlerError> {
    let performed_on = NaiveDate::parse_from_str(&form.performed_on, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Некорректная дата выполнения работы".into(),
        )
    })?;

    let department = form
        .department
        .trim()
        .to_owned();
    let workshop = form
        .workshop
        .trim()
        .to_owned();

    if department.is_empty() || workshop.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Необходимо указать службу и цех".into(),
        ));
    }

    let equipment = match form.equipment.trim() {
        "" => None,
        value => Some(value.to_owned()),
    };

    let performers = form
        .performers
        .split([',', '\n'])
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect();

    let draft = WorkRecordDraft {
        performed_on,
        department,
        workshop,
        equipment,
        description: form.description,
        performers,
    };

    draft
        .validate()
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                format!("Некорректные данные: {error:?}"),
            )
        })?;

    let database_path = state.database_path;

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut conn = open_connection(&database_path).map_err(|e| e.to_string())?;

        save_work_record(&mut conn, &draft).map_err(|error| format!("{error:?}"))?;

        Ok(())
    })
    .await
    .map_err(internal_error)?
    .map_err(internal_error)?;

    Ok(Redirect::to("/"))
}
