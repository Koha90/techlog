CREATE TABLE IF NOT EXISTS work_records (
    id INTEGER PRIMARY KEY,
    performed_on TEXT NOT NULL,
    department TEXT NOT NULL,
    workshop TEXT NOT NULL,
    equipment TEXT,
    description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS work_record_performers (
    id INTEGER PRIMARY KEY,
    work_record_id INTEGER NOT NULL,
    name TEXT NOT NULL,

    FOREIGN KEY (work_record_id)
        REFERENCES work_records(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_work_record_performers_record
    ON work_record_performers(work_record_id);
