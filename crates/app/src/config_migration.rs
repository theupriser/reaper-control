//! Bringing an older config file up to the current schema version.

use serde_json::{Value, json};

use crate::app_config::SCHEMA_VERSION;
use crate::config_error::ConfigError;

/// Applies every migration from the file's version up to [`SCHEMA_VERSION`].
///
/// A file without `schema_version` is version 0. Add one step per version here when the layout changes.
///
/// # Errors
/// [`ConfigError::Newer`] for a file from a newer app, [`ConfigError::Invalid`] when the file is not an object.
pub fn migrate(mut file: Value) -> Result<Value, ConfigError> {
    let Some(fields) = file.as_object_mut() else {
        return Err(ConfigError::Invalid("the file must hold an object".into()));
    };
    let found = fields
        .get("schema_version")
        .and_then(Value::as_u64)
        .and_then(|version| u32::try_from(version).ok())
        .unwrap_or(0);
    if found > SCHEMA_VERSION {
        return Err(ConfigError::Newer {
            found,
            supported: SCHEMA_VERSION,
        });
    }
    if found < 1 {
        fields.insert("schema_version".into(), json!(1));
    }
    Ok(file)
}
