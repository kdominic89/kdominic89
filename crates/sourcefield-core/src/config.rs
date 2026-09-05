use std::{fs, path::Path};

use thiserror::Error;

use crate::Config;

/// Failure to read or parse a profile configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The requested file could not be read.
    #[error("failed to read {path}: {source}")]
    Read {
        /// Path requested by the caller.
        path: String,
        /// Underlying filesystem error.
        source: std::io::Error,
    },
    /// The file is not a valid typed TOML configuration.
    #[error("failed to parse TOML configuration: {0}")]
    Parse(#[from] toml::de::Error),
}

/// Read typed TOML configuration without mutating the source file.
///
/// Call [`crate::validate_config`] to validate semantic references after parsing.
pub fn load_config(path: impl AsRef<Path>) -> Result<Config, ConfigError> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.display().to_string(),
        source,
    })?;

    Ok(toml::from_str(&text)?)
}
