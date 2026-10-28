//! SQLite storage for technical work records.

use rusqlite::Connection;

mod employee_work;
mod employees;
mod work_records;

pub use employees::{CreateEmployeeError, create_employee, list_employees, set_employee_active};

pub use work_records::{
    SaveWorkError, WorkRecordFilter, find_work_record, list_work_records,
    list_work_records_filtered, save_work_record,
};

pub use employee_work::{SaveEmployeeWorkError, save_work_with_employees};

/// Initialize the SQLite connection and creates the initial schema.
///
/// Enables foreign key enforcement for the given connection.
/// Existing tables are preserved.
pub fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    let version: i64 = conn.pragma_query_value(None, "user_version", |row| {
        row.get(0)
    })?;

    if version >= 3 {
        return Ok(());
    }

    let tx = conn.unchecked_transaction()?;

    if version < 1 {
        tx.execute_batch(include_str!(
            "../../migrations/001_create_work_records.sql"
        ))?;
    }

    if version < 2 {
        tx.execute_batch(include_str!(
            "../../migrations/002_create_employees.sql"
        ))?;
    }

    if version < 3 {
        tx.execute_batch(include_str!(
            "../../migrations/003_link_work_performers.sql"
        ))?;
    }

    tx.pragma_update(None, "user_version", 3)?;

    tx.commit()
}

#[cfg(test)]
mod tests;
