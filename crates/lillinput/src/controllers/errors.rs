//! Errors related to controller.

use crate::events::errors::{LibinputError, ProcessorError};
use crate::events::ActionEvent;
use input::event::gesture::GestureSwipeEvent;
use thiserror::Error;

/// Errors raised during processing of events in the controller.
#[derive(Error, Debug)]
pub enum ControllerError {
    /// Unsupported finger count.
    #[error("unsupported finger count ({0})")]
    UnsupportedFingerCount(i32),

    /// Unsupported swipe event.
    #[error("unsupported swipe event ({:?})", .0)]
    UnsupportedSwipeEvent(GestureSwipeEvent),

    /// Event displacement is below threshold.
    #[error("event displacement is below threshold ({0})")]
    DisplacementBelowThreshold(f64),

    /// No actions registered for event.
    #[error("no actions registered for event {0}")]
    NoActionsRegistered(ActionEvent),

    /// Unknown error while polling for the file descriptor.
    #[error("unknown error while polling the file descriptor")]
    ProcessorError(#[from] ProcessorError),

    /// Unknown error while polling for the file descriptor.
    #[error("unknown error while polling the file descriptor")]
    LibinputError(#[from] LibinputError),
}
