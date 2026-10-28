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

#[test]
fn creates_and_deactivates_employee() {
    use crate::domain::employee::EmployeeDraft;

    let conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();

    let draft = EmployeeDraft {
        full_name: " Иванов И.И. ".into(),
        department: " КИПиА ".into(),
    };

    let employee = create_employee(&conn, &draft).unwrap();

    assert!(employee.id > 0);
    assert_eq!(employee.full_name, "Иванов И.И.");
    assert_eq!(employee.department, "КИПиА");
    assert!(employee.is_active);

    let updated = set_employee_active(&conn, employee.id, false).unwrap();

    assert!(updated);

    let employees = list_employees(&conn).unwrap();

    assert_eq!(employees.len(), 1);
    assert_eq!(employees[0].id, employee.id);
    assert!(!employees[0].is_active);
}

#[test]
fn rejects_invalid_employee_without_inserting() {
    use crate::domain::employee::EmployeeDraft;

    let conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();

    let draft = EmployeeDraft {
        full_name: " ".into(),
        department: "КИПиА".into(),
    };

    let result = create_employee(&conn, &draft);

    assert!(matches!(
        result,
        Err(CreateEmployeeError::Validation(_))
    ));

    assert!(
        list_employees(&conn)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn upgrades_existing_database_without_losing_works() {
    let conn = Connection::open_in_memory().unwrap();

    // Simulate an existing database created before migration 002.
    conn.execute_batch(include_str!(
        "../../migrations/001_create_work_records.sql"
    ))
    .unwrap();

    conn.execute_batch(
        "INSERT INTO work_records
         (performed_on, department, workshop, description)
         VALUES (
             '2026-10-24',
             'КИПиА',
             'ЦМЦ',
             'Замена датчика давления'
         );",
    )
    .unwrap();

    let id = conn.last_insert_rowid();

    // Migration must work on an existing database.
    init_db(&conn).unwrap();

    // Repeated initialization must not destroy existing data.
    init_db(&conn).unwrap();

    let work = find_work_record(&conn, id)
        .unwrap()
        .expect("existing work record must survive migration");

    assert_eq!(
        work.description,
        "Замена датчика давления"
    );

    let version: i64 = conn
        .pragma_query_value(None, "user_version", |row| {
            row.get(0)
        })
        .unwrap();

    assert_eq!(version, 3);
    assert!(
        list_employees(&conn)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn saves_work_selected_employes() {
    use crate::domain::{employee::EmployeeDraft, work_record::EmployeeWorkDraft};

    let mut conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();

    let first = create_employee(
        &conn,
        &EmployeeDraft {
            full_name: "Иванов И.И.".into(),
            department: "КИПиА".into(),
        },
    )
    .unwrap();

    let second = create_employee(
        &conn,
        &EmployeeDraft {
            full_name: "Петров П.П.".into(),
            department: "КИПиА".into(),
        },
    )
    .unwrap();

    let draft = EmployeeWorkDraft {
        performed_on: NaiveDate::from_ymd_opt(2026, 10, 27).unwrap(),
        department: "КИПиА".into(),
        workshop: "ЦМЦ".into(),
        equipment: None,
        description: "Замена датчика давления".into(),
        employee_ids: vec![first.id, second.id],
    };

    let record_id = save_work_with_employees(&mut conn, &draft).unwrap();

    let record = find_work_record(&conn, record_id)
        .unwrap()
        .expect("work record must exist");

    assert_eq!(
        record.performers,
        vec!["Иванов И.И.", "Петров П.П."]
    );

    let mut stmt = conn
        .prepare(
            "SELECT employee_id
             FROM work_record_performers
             WHERE work_record_id = ?1
             ORDER BY id",
        )
        .unwrap();

    let ids = stmt
        .query_map([record_id], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();

    assert_eq!(ids, vec![first.id, second.id]);
}

#[test]
fn rejects_work_with_inactive_employee() {
    use crate::domain::{employee::EmployeeDraft, work_record::EmployeeWorkDraft};

    let mut conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();

    let employee = create_employee(
        &conn,
        &EmployeeDraft {
            full_name: "Иванов И.И.".into(),
            department: "КИПиА".into(),
        },
    )
    .unwrap();

    set_employee_active(&conn, employee.id, false).unwrap();

    let draft = EmployeeWorkDraft {
        performed_on: NaiveDate::from_ymd_opt(2026, 10, 27).unwrap(),
        department: "КИПиА".into(),
        workshop: "ЦМЦ".into(),
        equipment: None,
        description: "Проверка оборудования".into(),
        employee_ids: vec![employee.id],
    };

    let result = save_work_with_employees(&mut conn, &draft);

    assert!(matches!(
        result,
        Err(SaveEmployeeWorkError::EmployeeUnavailable(_))
    ));

    assert!(
        list_work_records(&conn)
            .unwrap()
            .is_empty()
    );
}
