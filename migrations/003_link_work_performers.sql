-- Link performers to employees without changing legacy recodrs.
ALTER TABLE work_record_performers
ADD COLUMN employee_id INTEGER REFERENCES employees(id);

CREATE INDEX idx_work_performers_employee
    ON work_record_performers(employee_id);
