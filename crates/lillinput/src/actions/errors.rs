//! Errors related to [`actions`].
//!
//! [`actions`]: crate::actions

use thiserror::Error;

/// Errors raised during execution of an [`Action`].
///
/// [`Action`]: crate::actions::Action
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
