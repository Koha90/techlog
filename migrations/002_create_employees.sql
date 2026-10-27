-- Employees may exist without application accounts.
CREATE TABLE employees (
    id INTEGER PRIMARY KEY,
    full_name TEXT NOT NULL
        CHECK (length(trim(full_name)) > 0),
    department TEXT NOT NULL
        CHECK (length(trim(department)) > 0),
    is_active INTEGER NOT NULL DEFAULT 1
        CHECK (is_active IN (0, 1))
);

CREATE INDEX idx_employees_department
    ON employees(department, full_name);
