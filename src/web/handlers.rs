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
    domain::work_record::EmployeeWorkDraft,
    storage::{
        SaveEmployeeWorkError, list_employees, list_work_records_filtered, save_work_with_employees,
    },
};

use super::{
    AppState,
    filters::parse_journal_filter,
    forms::{JournalQuery, NewWorkForm},
    views::{JournalTemplate, RecordView},
};

pub(super) type HandlerError = (StatusCode, String);

/// Converts an internal error into an HTTP 500 response.
pub(super) fn internal_error(error: impl std::fmt::Display) -> HandlerError {
    eprintln!("TechLog error: {error}");

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Внутренняя ошибка сервера".into(),
    )
}

/// Opens a SQLite connection with foreign key enforcement.
pub(super) fn open_connection(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;

    conn.busy_timeout(Duration::from_secs(5))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    Ok(conn)
}

/// Display work records matching the query parameters.
pub(super) async fn show_journal(
    State(state): State<AppState>,
    Query(filters): Query<JournalQuery>,
) -> Result<Html<String>, HandlerError> {
    let filter = parse_journal_filter(&filters)?;

    if let (Some(form), Some(to)) = (filter.from, filter.to)
        && form > to
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Начальная дата позже конечной".into(),
        ));
    }

    let database_path = state.database_path;

    let (works, employees) = tokio::task::spawn_blocking(move || {
        let conn = open_connection(&database_path)?;

        let works = list_work_records_filtered(&conn, &filter)?;
        let employees = list_employees(&conn)?;

        Ok::<_, rusqlite::Error>((works, employees))
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
        employees: employees
            .into_iter()
            .filter(|employee| employee.is_active)
            .collect(),
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

    let employee_ids = form
        .employee_ids
        .split(',')
        .map(str::trim)
        .map(str::parse::<i64>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Некорректный список исполнителей".into(),
            )
        })?;

    let equipment = match form.equipment.trim() {
        "" => None,
        value => Some(value.to_owned()),
    };

    let draft = EmployeeWorkDraft {
        performed_on,
        department,
        workshop,
        equipment,
        description: form.description,
        employee_ids,
    };

    let database_path = state.database_path;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = open_connection(&database_path).map_err(SaveEmployeeWorkError::Database)?;

        save_work_with_employees(&mut conn, &draft)
    })
    .await
    .map_err(internal_error)?;

    match result {
        Ok(_) => Ok(Redirect::to("/")),

        Err(SaveEmployeeWorkError::Database(error)) => Err(internal_error(error)),

        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            format!("Не удалось сохранить работу: {error:?}"),
        )),
    }
}
