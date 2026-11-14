//! GUI error type for adapter boundaries and user-facing failures.

use std::fmt;

/// Error returned by GUI adapters that touch the filesystem or simulation.
#[derive(Debug)]
pub enum GuiError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Firmware(String),
    Simulation(String),
    Export(String),
    InvalidInput(String),
}

impl fmt::Display for GuiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Json(error) => write!(formatter, "JSON error: {error}"),
            Self::Firmware(message) => write!(formatter, "firmware error: {message}"),
            Self::Simulation(message) => write!(formatter, "simulation error: {message}"),
            Self::Export(message) => write!(formatter, "export error: {message}"),
            Self::InvalidInput(message) => write!(formatter, "invalid input: {message}"),
        }
    }
}

impl std::error::Error for GuiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Firmware(_) | Self::Simulation(_) | Self::Export(_) | Self::InvalidInput(_) => {
                None
            }
        }
    }
}

impl From<std::io::Error> for GuiError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for GuiError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

/// Result alias for GUI adapters.
pub type GuiResult<T> = Result<T, GuiError>;

/// Validates a probe sampling interval.
pub fn validate_sample_every(every: u64) -> GuiResult<u64> {
    if every == 0 {
        return Err(GuiError::InvalidInput(
            "sample interval must be positive".to_string(),
        ));
    }
    Ok(every)
}

#[cfg(test)]
mod tests {
    use super::{GuiError, validate_sample_every};

    #[test]
    fn error_display_names_the_boundary() {
        assert_eq!(
            GuiError::Firmware("bad hex".to_string()).to_string(),
            "firmware error: bad hex"
        );
        assert_eq!(
            GuiError::InvalidInput("empty".to_string()).to_string(),
            "invalid input: empty"
        );
    }

    #[test]
    fn io_errors_convert() {
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let gui = GuiError::from(error);
        assert!(gui.to_string().contains("missing"));
    }

    #[test]
    fn sample_interval_rejects_zero() {
        assert!(validate_sample_every(0).is_err());
        assert_eq!(validate_sample_every(100).expect("interval"), 100);
    }
}
