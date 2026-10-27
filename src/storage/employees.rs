//! SQLite persistence for employee directory records.

use rusqlite::{Connection, params};

use crate::domain::employee::{Employee, EmployeeDraft, EmployeeValidationError};

/// Errors that can occur while creating an employee.
#[derive(Debug)]
pub enum CreateEmployeeError {
    /// Invalid employee information.
    Validation(EmployeeValidationError),

    /// SQLite operation failed.
    Database(rusqlite::Error),
}

/// Validates and creates an employee.
///
/// Returns the complete persisted entity.
pub fn create_employee(
    conn: &Connection,
    draft: &EmployeeDraft,
) -> Result<Employee, CreateEmployeeError> {
    draft
        .validate()
        .map_err(CreateEmployeeError::Validation)?;

    let full_name = draft
        .full_name
        .trim()
        .to_owned();
    let department = draft
        .department
        .trim()
        .to_owned();

    conn.execute(
        "INSERT INTO employees (full_name, department)
         VALUES (?1, ?2)",
        params![full_name, department],
    )
    .map_err(CreateEmployeeError::Database)?;

    Ok(Employee {
        id: conn.last_insert_rowid(),
        full_name,
        department,
        is_active: true,
    })
}

/// Returns all employees, including inactive ones.
///
/// Active employees appear first.
pub fn list_employees(conn: &Connection) -> rusqlite::Result<Vec<Employee>> {
    let mut stmt = conn.prepare(
        "SELECT id, full_name, department, is_active
         FROM employees
         ORDER BY is_active DESC, department, full_name, id",
    )?;

    let employees = stmt.query_map([], |row| {
        Ok(Employee {
            id: row.get(0)?,
            full_name: row.get(1)?,
            department: row.get(2)?,
            is_active: row.get(3)?,
        })
    })?;

    employees.collect()
}

/// Updates the active status of an employee.
///
/// Returns `false` if the employee does not exist.
pub fn set_employee_active(conn: &Connection, id: i64, active: bool) -> rusqlite::Result<bool> {
    let affected = conn.execute(
        "UPDATE employees
         SET is_active = ?1
         WHERE id = ?2",
        params![active, id],
    )?;

    Ok(affected == 1)
}
