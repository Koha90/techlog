//! Entry point for the TechLog HTTP server.

use std::path::PathBuf;

use rusqlite::Connection;
use techlog::{storage::init_db, web};

/// Initialize SQLite and starts the local HTTP server.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_path = PathBuf::from("techlog.db");

    let conn = Connection::open(&database_path)?;

    init_db(&conn)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;

    drop(conn);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("TechLog: http://127.0.0.1:3000");

    axum::serve(
        listener,
        web::router(database_path),
    )
    .await?;

    Ok(())
}
