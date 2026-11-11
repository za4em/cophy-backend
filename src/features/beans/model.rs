use serde::{Deserialize, Serialize};

use crate::core::errors::AppError;

#[derive(Debug, Deserialize)]
pub(crate) struct BeanDraft {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) origin: String,
    #[serde(default)]
    pub(crate) roaster: String,
    #[serde(default)]
    pub(crate) process: String,
    #[serde(default)]
    pub(crate) elevation: String,
    #[serde(default)]
    pub(crate) tasting_notes: Vec<String>,
}

impl BeanDraft {
    pub(crate) fn normalized(mut self) -> Self {
        self.name = self.name.trim().to_owned();
        self.origin = self.origin.trim().to_owned();
        self.roaster = self.roaster.trim().to_owned();
        self.process = self.process.trim().to_owned();
        self.elevation = self.elevation.trim().to_owned();
        self.tasting_notes = self
            .tasting_notes
            .into_iter()
            .map(|note| note.trim().to_owned())
            .filter(|note| !note.is_empty())
            .collect();
        self
    }

    pub(crate) fn require_name(self) -> Result<Self, AppError> {
        let draft = self.normalized();
        if draft.name.is_empty() {
            return Err(AppError::NameRequired);
        }
        Ok(draft)
    }
}

#[derive(Serialize)]
pub(crate) struct Bean {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) origin: String,
    pub(crate) roaster: String,
    pub(crate) process: String,
    pub(crate) elevation: String,
    pub(crate) tasting_notes: Vec<String>,
    pub(crate) image_url: Option<String>,
}

pub(crate) fn encode_notes(notes: &[String]) -> String {
    serde_json::to_string(notes).unwrap_or_else(|_| "[]".to_owned())
}

pub(crate) fn decode_notes(raw: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(raw.trim()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_note_lists_and_string_elevation() {
        let draft: BeanDraft = serde_json::from_str(
            r#"{"name":" Kieni ","origin":"Kenya","roaster":"","process":"Washed","elevation":"1,750 m","tasting_notes":[" blackcurrant ","", "grapefruit"]}"#,
        )
        .unwrap();
        let draft = draft.require_name().unwrap();
        assert_eq!(draft.name, "Kieni");
        assert_eq!(draft.elevation, "1,750 m");
        assert_eq!(
            draft.tasting_notes,
            vec!["blackcurrant".to_owned(), "grapefruit".to_owned()]
        );
    }

    #[test]
    fn rejects_a_note_string() {
        assert!(
            serde_json::from_str::<BeanDraft>(
                r#"{"name":"Kieni","tasting_notes":"blackcurrant"}"#,
            )
            .is_err()
        );
    }

    #[test]
    fn blank_name_is_required() {
        let draft: BeanDraft = serde_json::from_str(r#"{"name":"   "}"#).unwrap();
        assert_eq!(draft.require_name().unwrap_err(), AppError::NameRequired);
    }

    #[test]
    fn corrupt_notes_decode_to_an_empty_list() {
        assert!(decode_notes("{").is_empty());
        assert_eq!(decode_notes(r#" ["a"] "#), vec!["a".to_owned()]);
    }
}
