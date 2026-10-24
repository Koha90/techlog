//! View models used by HTML templates.

use askama::Template;

use crate::domain::work_record::WorkRecord;

use super::forms::JournalQuery;

/// Template context for the work journal page.
#[derive(Template)]
#[template(path = "index.html")]
pub(super) struct JournalTemplate {
    pub today: String,
    pub records: Vec<RecordView>,
    pub has_records: bool,
    pub filters: JournalQuery,
}

/// Display representation of a saved work record.
pub(super) struct RecordView {
    date: String,
    department: String,
    workshop: String,
    equipment: String,
    description: String,
    performers: String,
}

impl From<WorkRecord> for RecordView {
    fn from(record: WorkRecord) -> Self {
        Self {
            date: record
                .performed_on
                .to_string(),
            department: record.department,
            workshop: record.workshop,
            equipment: record
                .equipment
                .unwrap_or_else(|| "Не указано".into()),
            description: record.description,
            performers: record.performers.join(", "),
        }
    }
}
