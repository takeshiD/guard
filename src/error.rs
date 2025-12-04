//! Error types for the guard tool

use thiserror::Error;
use std::path::PathBuf;

/// Error type used throughout guard
#[derive(Error, Debug)]
pub enum GuardError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Git operation error
    #[error("Git error: {0}")]
    Git(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// Invalid line range
    #[error("Invalid line range: {start}-{end}")]
    InvalidRange { start: usize, end: usize },

    /// Agent command not found
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// Violation occurred in strict mode
    #[error("Violation in strict mode")]
    StrictModeViolation,

    /// I/O error
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Any other error wrapped by anyhow
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Convenient result type used in this crate
pub type Result<T> = std::result::Result<T, GuardError>;
