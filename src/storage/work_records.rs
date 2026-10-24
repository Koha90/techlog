//! Persistence operations for technical work records.

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::domain::work_record::{WorkRecord, WorkRecordDraft, WorkRecordError};
use chrono::NaiveDate;

/// Errors that can occur when saving a work record.
#[derive(Debug)]
pub enum SaveWorkError {
    /// The work record draft failed domain validation.
    Validation(WorkRecordError),

    /// A SQLite operation failed.
    Database(rusqlite::Error),
}

/// Optional search criteria for work records.
#[derive(Debug, Default, Clone)]
pub struct WorkRecordFilter {
    /// Inclusive start date.
    pub from: Option<NaiveDate>,

    /// Inclusive end date.
    pub to: Option<NaiveDate>,

    /// Exact department name.
    pub department: Option<String>,

    /// Exact workshop name.
    pub workshop: Option<String>,

    /// Case-sensitive substring of a performer's name.
    pub performer: Option<String>,
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

/// Converts a database row into a work record.
///
/// Performer names are loaded separately.
fn record_from_row(row: &Row<'_>) -> rusqlite::Result<WorkRecord> {
    Ok(WorkRecord {
        id: row.get(0)?,
        performed_on: row.get(1)?,
        department: row.get(2)?,
        workshop: row.get(3)?,
        equipment: row.get(4)?,
        description: row.get(5)?,
        performers: Vec::new(),
    })
}

/// Finds a work record by its database identifier.
///
/// Returns `None` if the record does not exist.
/// Loads all performers associated with the record.
pub fn find_work_record(conn: &Connection, id: i64) -> rusqlite::Result<Option<WorkRecord>> {
    let record = conn
        .query_row(
            "SELECT id, performed_on, department,
                    workshop, equipment, description
             FROM work_records
             WHERE id = ?1",
            [id],
            record_from_row,
        )
        .optional()?;

    let Some(mut record) = record else {
        return Ok(None);
    };

    let mut stmt = conn.prepare(
        "SELECT name
         FROM work_record_performers
         WHERE work_record_id = ?1
         ORDER BY id",
    )?;

    let performers = stmt.query_map([id], |row| row.get(0))?;

    record.performers = performers.collect::<rusqlite::Result<Vec<String>>>()?;

    Ok(Some(record))
}

/// Lists all work records with their performers.
///
/// Records are ordered by work date, newest first.
/// Uses a single SQL query to avoid loading performers
/// through a separate query for every record.
pub fn list_work_records(conn: &Connection) -> rusqlite::Result<Vec<WorkRecord>> {
    list_work_records_filtered(conn, &WorkRecordFilter::default())
}

/// List work records matching the specified search criteria.
///
/// A performer filter selects matching work records while still
/// returning all performes associated with those records.
pub fn list_work_records_filtered(
    conn: &Connection,
    filter: &WorkRecordFilter,
) -> rusqlite::Result<Vec<WorkRecord>> {
    let mut stmt = conn.prepare(
        "SELECT
            w.id,
            w.performed_on,
            w.department,
            w.workshop,
            w.equipment,
            w.description,
            p.name
         FROM work_records AS w
         LEFT JOIN work_record_performers AS p
             ON p.work_record_id = w.id
         WHERE
             (?1 IS NULL OR w.performed_on >= ?1)
             AND (?2 IS NULL OR w.performed_on <= ?2)
             AND (?3 IS NULL OR w.department = ?3)
             AND (?4 IS NULL OR w.workshop = ?4)
             AND (
                 ?5 IS NULL
                 OR EXISTS (
                     SELECT 1
                     FROM work_record_performers AS matched
                     WHERE matched.work_record_id = w.id
                       AND instr(matched.name, ?5) > 0
                 )
             )
         ORDER BY
             w.performed_on DESC,
             w.id DESC,
             p.id ASC",
    )?;

    let from = filter
        .from
        .map(|date| date.to_string());
    let to = filter
        .to
        .map(|date| date.to_string());

    let mut rows = stmt.query(params![
        from.as_deref(),
        to.as_deref(),
        filter.department.as_deref(),
        filter.workshop.as_deref(),
        filter.performer.as_deref(),
    ])?;

    let mut records: Vec<WorkRecord> = Vec::new();

    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;

        if records
            .last()
            .is_none_or(|record| record.id != id)
        {
            records.push(record_from_row(row)?);
        }

        if let Some(name) = row.get::<_, Option<String>>(6)?
            && let Some(record) = records.last_mut()
        {
            record.performers.push(name);
        }
    }

    Ok(records)
}
