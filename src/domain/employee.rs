//! Domain models and validation rules for employees.

use askama::Result;

/// An employee registered in the company directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Employee {
    /// Unique internal identifier.
    pub id: i64,

    /// Employee's full name.
    pub full_name: String,

    /// Department the employee belongs to.
    pub department: String,

    /// Whether the employee is currently active.
    pub is_active: bool,
}

/// Input data used to create an employee.
#[derive(Debug, Clone)]
pub struct EmployeeDraft {
    /// Employee's full name.
    pub full_name: String,

    /// Employee's department.
    pub department: String,
}

/// Errors produced by employee validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmployeeValidationError {
    EmptyFullName,
    EmptyDepartment,
    FullNameTooLong,
    DepartmentTooLong,
}

impl EmployeeDraft {
    /// Validates the required employee information.
    ///
    /// Names are limited to 120 Unicode characters and
    /// department names to 100 Unicode characters.
    pub fn validate(&self) -> Result<(), EmployeeValidationError> {
        let name = self.full_name.trim();
        let department = self.department.trim();

        if name.is_empty() {
            return Err(EmployeeValidationError::EmptyFullName);
        }

        if name.chars().count() > 120 {
            return Err(EmployeeValidationError::FullNameTooLong);
        }

        if department.is_empty() {
            return Err(EmployeeValidationError::EmptyDepartment);
        }

        if department.chars().count() > 100 {
            return Err(EmployeeValidationError::DepartmentTooLong);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_employee_with_blank_name() {
        let draft = EmployeeDraft {
            full_name: "    ".into(),
            department: "КИПиА".into(),
        };

        assert_eq!(
            draft.validate(),
            Err(EmployeeValidationError::EmptyFullName)
        )
    }

    #[test]
    fn accept_valid_employee() {
        let draft = EmployeeDraft {
            full_name: "Иванов И.И.".into(),
            department: "КИПиА".into(),
        };

        assert_eq!(draft.validate(), Ok(()));
    }
}
