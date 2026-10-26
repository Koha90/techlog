//! Validation and conversion of journal query parameters.

use axum::http::StatusCode;
use chrono::NaiveDate;

use crate::storage::WorkRecordFilter;

use super::forms::JournalQuery;

type FilterError = (StatusCode, String);

/// Converts HTTP query parameters into database search criteria.
///
/// Returns HTTP 400 for invalid dates or recersed date ranges.
pub(super) fn parse_journal_filter(query: &JournalQuery) -> Result<WorkRecordFilter, FilterError> {
    let filter = WorkRecordFilter {
        from: parse_date(&query.from)?,
        to: parse_date(&query.to)?,
        department: optional_text(&query.department),
        workshop: optional_text(&query.workshop),
        performer: optional_text(&query.performer),
    };

    if let (Some(from), Some(to)) = (filter.from, filter.to)
        && from > to
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Начальная дата позже конечной".into(),
        ));
    }

    Ok(filter)
}

/// Parses an optional date in YYYY-MM-DD format.
fn parse_date(value: &str) -> Result<Option<NaiveDate>, FilterError> {
    let value = value.trim();

    if value.is_empty() {
        return Ok(None);
    }

    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Некорректная дата в фидьтре".into(),
            )
        })
}

/// Normalizes an optional text search parameter.
fn optional_text(value: &str) -> Option<String> {
    let value = value.trim();

    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}
