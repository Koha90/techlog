use chrono::NaiveDate;
use rusqlite::Connection;

use crate::domain::work_record::WorkRecordDraft;

use super::*;

#[test]
fn saves_work_with_multiple_performers() {
    let mut conn = Connection::open_in_memory().unwrap();

    init_db(&conn).unwrap();

    let draft = WorkRecordDraft {
        performed_on: NaiveDate::from_ymd_opt(2026, 10, 21).unwrap(),
        department: "КИПиА".into(),
        workshop: "ЦМЦ".into(),
        equipment: None,
        description: "Замена датчика даваления".into(),
        performers: vec!["Нохрин А.Ю.".into(), "Иванов И.И.".into()],
    };

    let record_id = save_work_record(&mut conn, &draft).expect("work record must be created");

    let records_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM work_records",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(records_count, 1);

    let performers_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM work_record_performers
                 WHERE work_record_id = ?1",
            [record_id],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(performers_count, 2);
}

#[test]
fn rolls_back_work_when_performer_insert_fails() {
    let mut conn = Connection::open_in_memory().unwrap();

    init_db(&conn).unwrap();

    conn.execute_batch(
        "CREATE TRIGGER reject_performer
             BEFORE INSERT ON work_record_performers
             WHEN NEW.name = 'Ошибка'
             BEGIN
                 SELECT RAISE(ABORT, 'Test failure');
             END;",
    )
    .unwrap();

    let draft = WorkRecordDraft {
        performed_on: NaiveDate::from_ymd_opt(2026, 10, 21).unwrap(),
        department: "КИПиА".into(),
        workshop: "ЦМЦ".into(),
        equipment: None,
        description: "Замена датчика даваления".into(),
        performers: vec!["Иванов".into(), "Петров".into(), "Ошибка".into()],
    };

    let result = save_work_record(&mut conn, &draft);

    assert!(result.is_err());

    assert!(matches!(
        result,
        Err(SaveWorkError::Database(_))
    ));

    let records_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM work_records",
            [],
            |row| row.get(0),
        )
        .unwrap();

    let performers_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM work_record_performers",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(records_count, 0);
    assert_eq!(performers_count, 0);
}
