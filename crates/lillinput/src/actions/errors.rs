//! Errors related to events.

use thiserror::Error;

/// Errors related to `Actions`.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ActionError {
    /// Command execution resulted in error.
    #[error("{type_}: command execution resulted in error: {message}")]
    ExecutionError {
        /// Action type.
        type_: String,
        /// Command error message.
        message: String,
    },
}
