use chrono::NaiveDate;

/// A technical work record before persistence.
///
/// Contains the work derails and at least one performer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkRecordDraft {
    pub performed_on: NaiveDate,
    pub department: String,
    pub workshop: String,
    pub equipment: Option<String>,
    pub description: String,
    pub performers: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WorkRecordError {
    EmptyDescription,
    MissingPerformers,
    EmptyPerformerName,
}

impl WorkRecordDraft {
    /// Validates the work description and performer list.
    ///
    /// Returns an error if the description is blank,
    /// no performers are provided, or any performer
    /// has a blank name.
    pub fn validate(&self) -> Result<(), WorkRecordError> {
        if self
            .description
            .trim()
            .is_empty()
        {
            return Err(WorkRecordError::EmptyDescription);
        }

        if self.performers.is_empty() {
            return Err(WorkRecordError::MissingPerformers);
        }

        if self
            .performers
            .iter()
            .any(|name| name.trim().is_empty())
        {
            return Err(WorkRecordError::EmptyPerformerName);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_empty_description() {
        let draft = WorkRecordDraft {
            performed_on: NaiveDate::from_ymd_opt(2026, 10, 20).unwrap(),
            department: "КИПиА".into(),
            workshop: "Цельномолочный Цех (ЦМЦ)".into(),
            equipment: None,
            description: "     ".into(),
            performers: vec!["Нохрин А.Ю.".into()],
        };

        assert_eq!(
            draft.validate(),
            Err(WorkRecordError::EmptyDescription)
        );
    }

    #[test]
    fn accepts_valid_description() {
        let draft = WorkRecordDraft {
            performed_on: NaiveDate::from_ymd_opt(2026, 10, 20).unwrap(),
            department: "КИПиА".into(),
            workshop: "Цельномолочный Цех (ЦМЦ)".into(),
            equipment: None,
            description: "Заменён датчик давления PT-101".into(),
            performers: vec!["Нохрин А.Ю.".into()],
        };

        assert_eq!(draft.validate(), Ok(()));

        assert_eq!(
            draft.description,
            "Заменён датчик давления PT-101",
        );
    }

    #[test]
    fn rejects_work_without_performers() {
        let draft = WorkRecordDraft {
            performed_on: NaiveDate::from_ymd_opt(2026, 10, 20).unwrap(),
            department: "КИПиА".into(),
            workshop: "ЦМЦ".into(),
            equipment: None,
            description: "Замена датчика давления".into(),
            performers: vec![],
        };

        assert_eq!(
            draft.validate(),
            Err(WorkRecordError::MissingPerformers)
        );
    }

    #[test]
    fn rejects_work_with_empty_name_performer() {
        let draft = WorkRecordDraft {
            performed_on: NaiveDate::from_ymd_opt(2026, 10, 20).unwrap(),
            department: "КИПиА".into(),
            workshop: "ЦМЦ".into(),
            equipment: None,
            description: "Замена датчика давления".into(),
            performers: vec!["Нохрин А.Ю.".into(), "   ".into()],
        };

        assert_eq!(
            draft.validate(),
            Err(WorkRecordError::EmptyPerformerName)
        )
    }
}
