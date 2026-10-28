//! Persistence of work records linked to directory employees.

use std::collections::HashSet;

use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::work_record::{EmployeeWorkDraft, WorkRecordDraft, WorkRecordError};

/// Errors produced while saving work with selected employees.
#[derive(Debug)]
pub enum SaveEmployeeWorkError {
    /// The work draft failed validation.
    Validation(WorkRecordError),

    /// Employee IDs are empty, invalid, or duplicated.
    InvalidSelection,

    /// An employee does not exist or inactive.
    EmployeeUnavailable(i64),

    /// A SQLite operation failed.
    Database(rusqlite::Error),
}

/// Saves a work record and its selected employees atomically.
///
/// Resolves active employees inside the transaction.
/// Stores both employee IDs and historical name snapshots.
pub fn save_work_with_employees(
    conn: &mut Connection,
    draft: &EmployeeWorkDraft,
) -> Result<i64, SaveEmployeeWorkError> {
    if draft.employee_ids.is_empty() {
        return Err(SaveEmployeeWorkError::Validation(
            WorkRecordError::MissingPerformers,
        ));
    }

    let mut unique = HashSet::new();

    if draft.employee_ids.len() > 20
        || draft
            .employee_ids
            .iter()
            .any(|&id| id <= 0 || !unique.insert(id))
    {
        return Err(SaveEmployeeWorkError::InvalidSelection);
    }

    let tx = conn
        .transaction()
        .map_err(SaveEmployeeWorkError::Database)?;

    let mut performers = Vec::with_capacity(draft.employee_ids.len());

    for &employee_id in &draft.employee_ids {
        let name: Option<String> = tx
            .query_row(
                "SELECT full_name
                 FROM employees
                 WHERE id = ?1 AND is_active = 1",
                [employee_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(SaveEmployeeWorkError::Database)?;

        let name = name.ok_or(SaveEmployeeWorkError::EmployeeUnavailable(employee_id))?;

        performers.push((employee_id, name));
    }

    let work = WorkRecordDraft {
        performed_on: draft.performed_on,
        department: draft
            .department
            .trim()
            .to_owned(),
        workshop: draft
            .workshop
            .trim()
            .to_owned(),
        equipment: draft.equipment.clone(),
        description: draft.description.clone(),
        performers: performers
            .iter()
            .map(|(_, name)| name.clone())
            .collect(),
    };

    work.validate()
        .map_err(SaveEmployeeWorkError::Validation)?;

    tx.execute(
        "INSERT INTO work_records
         (performed_on, department, workshop, equipment, description)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            work.performed_on.to_string(),
            work.department,
            work.workshop,
            work.equipment,
            work.description.trim(),
        ],
    )
    .map_err(SaveEmployeeWorkError::Database)?;

    let record_id = tx.last_insert_rowid();

    for (employee_id, name) in performers {
        tx.execute(
            "INSERT INTO work_record_performers
             (work_record_id, employee_id, name)
             VALUES (?1, ?2, ?3)",
            params![record_id, employee_id, name],
        )
        .map_err(SaveEmployeeWorkError::Database)?;
    }

    tx.commit()
        .map_err(SaveEmployeeWorkError::Database)?;

    Ok(record_id)
}
