//! Reading a v1 `setlists/<projectId>.json`.

use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::import_error::ImportError;
use crate::legacy_setlist::LegacySetlist;

/// What a v1 setlist file holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LegacyFile {
    /// The setlists.
    pub setlists: Vec<LegacySetlist>,
    /// The setlist v1 had selected, if it recorded one.
    pub selected: Option<String>,
}

#[derive(Deserialize)]
struct Metadata {
    #[serde(rename = "selectedSetlistId")]
    selected: Option<String>,
}

impl LegacyFile {
    /// Reads the file; v1 wrote either a bare array of setlists or an object with `setlists` and `metadata`.
    ///
    /// # Errors
    /// [`ImportError`] when the file cannot be read or is not a v1 setlist file.
    pub fn read(file: &Path) -> Result<Self, ImportError> {
        Self::parse(&std::fs::read_to_string(file)?)
    }

    /// Understands the text of a v1 setlist file.
    ///
    /// # Errors
    /// [`ImportError::Parse`] when the text is not a v1 setlist file.
    pub fn parse(text: &str) -> Result<Self, ImportError> {
        match serde_json::from_str::<Value>(text)? {
            list @ Value::Array(_) => Ok(Self {
                setlists: serde_json::from_value(list)?,
                selected: None,
            }),
            Value::Object(mut fields) => {
                let setlists = fields
                    .remove("setlists")
                    .unwrap_or(Value::Array(Vec::new()));
                let metadata = fields.remove("metadata").unwrap_or(Value::Null);
                Ok(Self {
                    setlists: serde_json::from_value(setlists)?,
                    selected: serde_json::from_value::<Option<Metadata>>(metadata)?
                        .and_then(|metadata| metadata.selected),
                })
            }
            _ => Err(ImportError::NotASetlistFile),
        }
    }
}
