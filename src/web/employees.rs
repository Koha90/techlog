//! HTTP handlers for the employee directory.

use askama::{Result, Template};
use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Redirect},
};

use crate::{
    domain::employee::{Employee, EmployeeDraft},
    storage::{create_employee as insert_employee, list_employees, set_employee_active},
};

use super::{
    AppState,
    forms::{EmployeeStatusForm, NewEmployeeForm},
    handlers::{HandlerError, internal_error, open_connection},
};

/// HTML view of the employee directory.
#[derive(Template)]
#[template(path = "employees.html")]
struct EmployeesTemplate {
    employees: Vec<Employee>,
    has_employees: bool,
}

/// Display registered employees.
pub(super) async fn show_employees(
    State(state): State<AppState>,
) -> Result<Html<String>, HandlerError> {
    let database_path = state.database_path;

    let employees = tokio::task::spawn_blocking(move || {
        let conn = open_connection(&database_path)?;
        list_employees(&conn)
    })
    .await
    .map_err(internal_error)?
    .map_err(internal_error)?;

    let page = EmployeesTemplate {
        has_employees: !employees.is_empty(),
        employees,
    };

    let html = page
        .render()
        .map_err(internal_error)?;

    Ok(Html(html))
}

/// Validates and registers a new employee.
pub(super) async fn create_employee(
    State(state): State<AppState>,
    Form(form): Form<NewEmployeeForm>,
) -> Result<Redirect, HandlerError> {
    let draft = EmployeeDraft {
        full_name: form.full_name,
        department: form.department,
    };

    draft
        .validate()
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                format!("Некорректные данные сотрудника: {error:?}"),
            )
        })?;

    let database_path = state.database_path;

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let conn = open_connection(&database_path).map_err(|error| error.to_string())?;

        insert_employee(&conn, &draft).map_err(|error| format!("{error:?}"))?;

        Ok(())
    })
    .await
    .map_err(internal_error)?
    .map_err(internal_error)?;

    Ok(Redirect::to("/employees"))
}

/// Activates or deactivates an employee.
pub(super) async fn update_employee_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<EmployeeStatusForm>,
) -> Result<Redirect, HandlerError> {
    let database_path = state.database_path;

    let updated = tokio::task::spawn_blocking(move || {
        let conn = open_connection(&database_path)?;
        set_employee_active(&conn, id, form.active)
    })
    .await
    .map_err(internal_error)?
    .map_err(internal_error)?;

    if !updated {
        return Err((
            StatusCode::NOT_FOUND,
            "Сотрудник не найден".into(),
        ));
    }

    Ok(Redirect::to("/employees"))
}
