//! Microsoft Word report generation.

use std::io::{self, Cursor};

use chrono::NaiveDate;
use docx_rs::{
    Docx, PageMargin, PageOrientationType, Paragraph, Run, RunFonts, Table, TableCell, TableRow,
};

use crate::{domain::work_record::WorkRecord, storage::WorkRecordFilter};

/// Generate a Word report from already filtered work records.
///
/// Returns a complete DOCX document as bytes.
/// Does not read from the database or write to disk.
pub fn generate_work_report(
    records: &[WorkRecord],
    filter: &WorkRecordFilter,
    generated_on: NaiveDate,
) -> io::Result<Vec<u8>> {
    let fonts = RunFonts::new()
        .ascii("Arial")
        .hi_ansi("Arial")
        .cs("Arial");

    let mut document = Docx::new()
        .page_size(16838, 11906)
        .page_orient(PageOrientationType::Landscape)
        .page_margin(
            PageMargin::new()
                .top(850)
                .bottom(850)
                .left(850)
                .right(850),
        )
        .default_fonts(fonts)
        .default_size(20)
        .add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text("ЖУРНАЛ ВЫПОЛНЕННЫХ РАБОТ")
                    .bold()
                    .size(28),
            ),
        )
        .add_paragraph(text_paragraph(format!(
            "Дата формирования: {}",
            generated_on.format("%d.%m.%Y"),
        )))
        .add_paragraph(text_paragraph(format!(
            "Период: {}",
            format_period(filter),
        )))
        .add_paragraph(text_paragraph(format!(
            "Служба: {}",
            filter
                .department
                .as_deref()
                .unwrap_or("Все"),
        )))
        .add_paragraph(text_paragraph(format!(
            "Цех: {}",
            filter
                .workshop
                .as_deref()
                .unwrap_or("Все"),
        )))
        .add_paragraph(text_paragraph(format!(
            "Исполнитель: {}",
            filter
                .performer
                .as_deref()
                .unwrap_or("Все"),
        )))
        .add_paragraph(text_paragraph(format!(
            "Всего работ: {}",
            records.len(),
        )));

    if records.is_empty() {
        document = document.add_paragraph(text_paragraph(
            "За выбранный период работы не найдены.",
        ));
    } else {
        document = document.add_table(build_table(records));
    }

    let mut output = Cursor::new(Vec::new());

    document
        .pack(&mut output)
        .map_err(io::Error::other)?;

    Ok(output.into_inner())
}

/// Builds a paragraph containing plain text.
fn text_paragraph(text: impl Into<String>) -> Paragraph {
    Paragraph::new().add_run(Run::new().add_text(text.into()))
}

/// Formats the inclusive report date range.
fn format_period(filter: &WorkRecordFilter) -> String {
    let from = filter
        .from
        .map(|date| {
            date.format("%d.%m.%Y")
                .to_string()
        })
        .unwrap_or_else(|| "Без ограничения".into());

    let to = filter
        .to
        .map(|date| {
            date.format("%d.%m.%Y")
                .to_string()
        })
        .unwrap_or_else(|| "Без ограничения".into());

    format!("{from} — {to}")
}

/// Builds the table containing completed work records.
fn build_table(recrords: &[WorkRecord]) -> Table {
    let header = TableRow::new(vec![
        table_cell("Дата", true),
        table_cell("Служба", true),
        table_cell("Цех", true),
        table_cell("Оборудование", true),
        table_cell("Выполненные работы", true),
        table_cell("Исполнители", true),
    ]);

    let mut table = Table::new(vec![header]).set_grid(vec![
        1450, 1550, 1550, 1850, 5850, 2800,
    ]);

    for record in recrords {
        let date = record
            .performed_on
            .format("%d.%m.%Y")
            .to_string();

        let equipment = record
            .equipment
            .as_deref()
            .unwrap_or("Не указано");

        let performers = record.performers.join(", ");

        table = table.add_row(TableRow::new(vec![
            table_cell(&date, false),
            table_cell(&record.department, false),
            table_cell(&record.workshop, false),
            table_cell(equipment, false),
            table_cell(&record.description, false),
            table_cell(&performers, false),
        ]));
    }

    table
}

/// Creates a Word table cell while preserving text line breaks.
fn table_cell(text: &str, bold: bool) -> TableCell {
    let mut cell = TableCell::new();

    for line in text.split('\n') {
        let mut run = Run::new()
            .add_text(line.trim_end_matches('\r'))
            .size(18);

        if bold {
            run = run.bold();
        }

        cell = cell.add_paragraph(Paragraph::new().add_run(run));
    }

    cell
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use chrono::NaiveDate;
    use zip::ZipArchive;

    use super::*;

    #[test]
    fn generates_docx_with_work_records() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 24).unwrap();

        let records = vec![WorkRecord {
            id: 1,
            performed_on: date,
            department: "КИПиА".into(),
            workshop: "ЦМЦ".into(),
            equipment: Some("Пастеризатор".into()),
            description: "Замена датчика давления".into(),
            performers: vec!["Иванов И.И.".into(), "Петров П.П.".into()],
        }];

        let filter = WorkRecordFilter {
            from: Some(date),
            to: Some(date),
            department: Some("КИПиА".into()),
            ..Default::default()
        };

        let bytes = generate_work_report(&records, &filter, date).unwrap();

        let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();

        let mut xml = String::new();

        archive
            .by_name("word/document.xml")
            .unwrap()
            .read_to_string(&mut xml)
            .unwrap();

        assert!(xml.contains("ЖУРНАЛ ВЫПОЛНЕННЫХ РАБОТ"));
        assert!(xml.contains("Замена датчика давления"));
        assert!(xml.contains("Иванов И.И."));
        assert!(xml.contains("Петров П.П."));
        assert!(xml.contains("24.10.2026"));
        assert!(xml.contains("Всего работ: 1"));
    }

    #[test]
    fn generates_report_without_work_records() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 24).unwrap();

        let bytes = generate_work_report(
            &[],
            &WorkRecordFilter::default(),
            date,
        )
        .unwrap();

        let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();

        let mut xml = String::new();

        archive
            .by_name("word/document.xml")
            .unwrap()
            .read_to_string(&mut xml)
            .unwrap();

        assert!(xml.contains("Всего работ: 0"));
        assert!(xml.contains("работы не найдены"));
    }
}
