//! Input models for HTTP forms and query parameters.

use serde::Deserialize;

/// Submitted data for creating a technical work record.
#[derive(Deserialize)]
pub(super) struct NewWorkForm {
    pub performed_on: String,
    pub department: String,
    pub workshop: String,
    pub equipment: String,
    pub description: String,
    pub performers: String,
}

/// Query parameters for filtering the work journal.
///
/// Missing parameters default to empty strings.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(super) struct JournalQuery {
    pub from: String,
    pub to: String,
    pub department: String,
    pub workshop: String,
    pub performer: String,
}
