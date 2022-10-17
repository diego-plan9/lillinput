//! Errors related to events.

use crate::events::ActionEvents;
use thiserror::Error;

/// Errors raised during processing of events in the controller.
#[derive(Error, Debug, PartialEq)]
pub enum ActionControllerError {
    #[error("unsupported finger count ({0})")]
    UnsupportedFingerCount(i32),

    #[error("event displacement is below threshold ({0})")]
    DisplacementBelowThreshold(f64),

    #[error("no actions registered for event {0}")]
    NoActionsRegistered(ActionEvents),
}

/// Errors related to `Actions`.
#[derive(Error, Debug, PartialEq)]
pub enum ActionError {
    #[error("{kind}: command execution resulted in error: {message}")]
    ExecutionError { kind: String, message: String },
}
