//! Persistence operations for technical work records.

use rusqlite::{Connection, params};

use crate::domain::work_record::{WorkRecordDraft, WorkRecordError};

/// Errors that can occur when saving a work record.
#[derive(Debug)]
pub enum SaveWorkError {
    /// The work record draft failed domain validation.
    Validation(WorkRecordError),

    /// A SQLite operation failed.
    Database(rusqlite::Error),
}

/// Saves a work record and its performers in a single transaction.
///
/// Validates the draft before starting the transaction.
/// All changes are committed together or rolled back on failure.
///
/// Returns the ID of the newly created work record.
pub fn save_work_record(
    conn: &mut Connection,
    draft: &WorkRecordDraft,
) -> Result<i64, SaveWorkError> {
    draft
        .validate()
        .map_err(SaveWorkError::Validation)?;

    let tx = conn
        .transaction()
        .map_err(SaveWorkError::Database)?;

    tx.execute(
        "INSERT INTO work_records
         (performed_on, department, workshop, equipment, description)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            draft.performed_on.to_string(),
            &draft.department,
            &draft.workshop,
            draft.equipment.as_deref(),
            draft.description.trim(),
        ],
    )
    .map_err(SaveWorkError::Database)?;

    let record_id = tx.last_insert_rowid();

    for performer in &draft.performers {
        tx.execute(
            "INSERT INTO work_record_performers
             (work_record_id, name)
             VALUES (?1, ?2)",
            params![record_id, performer.trim()],
        )
        .map_err(SaveWorkError::Database)?;
    }

    tx.commit()
        .map_err(SaveWorkError::Database)?;

    Ok(record_id)
}
