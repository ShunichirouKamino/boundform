//! Custom error types for boundform.

use thiserror::Error;

/// Errors that can occur during boundform operations.
#[derive(Debug, Error)]
pub enum BoundformError {
    /// Failed to fetch HTML from a URL.
    #[error("failed to fetch URL: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Failed to read a local file.
    #[error("failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    /// Failed to parse config file.
    #[error("failed to parse config: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, BoundformError>;
