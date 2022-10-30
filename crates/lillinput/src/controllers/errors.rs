//! Errors related to controller.

use crate::events::ActionEvent;
use thiserror::Error;

/// Errors raised during processing of events in the controller.
#[derive(Error, Debug, PartialEq)]
pub enum ControllerError {
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
