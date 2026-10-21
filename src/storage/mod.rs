//! SQLite storage for technical work records.

use rusqlite::Connection;

mod work_records;

pub use work_records::{SaveWorkError, save_work_record};

/// Initialize the SQLite connection and creates the initial schema.
///
/// Enables foreign key enforcement for the given connection.
/// Existing tables are preserved.
pub fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    conn.execute_batch(include_str!(
        "../../migrations/001_create_work_records.sql"
    ))
}

#[cfg(test)]
mod tests;
