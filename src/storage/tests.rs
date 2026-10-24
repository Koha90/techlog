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

#[test]
fn reads_saved_work_with_performers() {
    let mut conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();

    let draft = WorkRecordDraft {
        performed_on: NaiveDate::from_ymd_opt(2026, 10, 21).unwrap(),
        department: "КИПиА".into(),
        workshop: "ЦМЦ".into(),
        equipment: Some("Пастеризатор".into()),
        description: "Замена датчика даваления".into(),
        performers: vec!["Иванов".into(), "Петров".into()],
    };

    let id = save_work_record(&mut conn, &draft).unwrap();

    let record = find_work_record(&conn, id)
        .unwrap()
        .expect("saved record must exist");

    assert_eq!(record.id, id);
    assert_eq!(
        record.description,
        draft.description
    );
    assert_eq!(record.equipment, draft.equipment);
    assert_eq!(record.performers, draft.performers);

    let records = list_work_records(&conn).unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0], record);
    assert!(
        find_work_record(&conn, id + 1)
            .unwrap()
            .is_none()
    );
}

#[test]
fn persists_work_after_database_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("techlog.db");

    let id = {
        let mut conn = Connection::open(&path).unwrap();
        init_db(&conn).unwrap();

        let draft = WorkRecordDraft {
            performed_on: NaiveDate::from_ymd_opt(2026, 10, 21).unwrap(),
            department: "КИПиА".into(),
            workshop: "ЦМЦ".into(),
            equipment: None,
            description: "Регулировка датчика".into(),
            performers: vec!["Иванов".into(), "Петров".into()],
        };

        save_work_record(&mut conn, &draft).unwrap()
    };

    let conn = Connection::open(&path).unwrap();
    let record = find_work_record(&conn, id)
        .unwrap()
        .expect("record must survive database reopen");

    assert_eq!(
        record.description,
        "Регулировка датчика"
    );
    assert_eq!(
        record.performers,
        vec!["Иванов", "Петров"]
    );
}

#[test]
fn filters_work_records_and_preserves_all_performers() {
    let mut conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();

    let drafts = [
        WorkRecordDraft {
            performed_on: NaiveDate::from_ymd_opt(2026, 10, 24).unwrap(),
            department: "КИПиА".into(),
            workshop: "ЦМЦ".into(),
            equipment: None,
            description: "Старая работа".into(),
            performers: vec!["Иванов".into()],
        },
        WorkRecordDraft {
            performed_on: NaiveDate::from_ymd_opt(2026, 10, 22).unwrap(),
            department: "КИПиА".into(),
            workshop: "ЦМЦ".into(),
            equipment: None,
            description: "Замена датчика".into(),
            performers: vec!["Иванов".into(), "Петров".into()],
        },
        WorkRecordDraft {
            performed_on: NaiveDate::from_ymd_opt(2026, 10, 22).unwrap(),
            department: "Электрики".into(),
            workshop: "ЦМЦ".into(),
            equipment: None,
            description: "Замена кабеля".into(),
            performers: vec!["Иванов".into()],
        },
    ];

    for draft in &drafts {
        save_work_record(&mut conn, draft).unwrap();
    }

    let filter = WorkRecordFilter {
        from: NaiveDate::from_ymd_opt(2026, 10, 21),
        to: NaiveDate::from_ymd_opt(2026, 10, 23),
        department: Some("КИПиА".into()),
        workshop: Some("ЦМЦ".into()),
        performer: Some("Иванов".into()),
    };

    let records = list_work_records_filtered(&conn, &filter).unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0].description,
        "Замена датчика"
    );
    assert_eq!(
        records[0].performers,
        vec!["Иванов", "Петров"]
    );
}
