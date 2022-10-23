//! Errors related to events.

use crate::events::ActionEvent;
use thiserror::Error;

/// Errors raised during processing of events in the controller.
#[derive(Error, Debug, PartialEq)]
pub enum ActionControllerError {
    /// Unsupported finger count.
    #[error("unsupported finger count ({0})")]
    UnsupportedFingerCount(i32),

    /// Event displacement is below threshold.
    #[error("event displacement is below threshold ({0})")]
    DisplacementBelowThreshold(f64),

    /// No actions registered for event.
    #[error("no actions registered for event {0}")]
    NoActionsRegistered(ActionEvent),
}

/// Errors related to `Actions`.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ActionError {
    /// Command execution resulted in error.
    #[error("{kind}: command execution resulted in error: {message}")]
    ExecutionError {
        /// Action kind.
        kind: String,
        /// Command error message.
        message: String,
    },
}
