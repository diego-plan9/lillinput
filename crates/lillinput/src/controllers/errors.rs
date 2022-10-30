//! Errors related to controller.

use crate::events::errors::{LibinputError, ProcessorError};
use crate::events::ActionEvent;
use thiserror::Error;

/// Errors raised during processing of events in the controller.
#[derive(Error, Debug)]
pub enum ControllerError {
    /// No actions registered for event.
    #[error("no actions registered for event {0}")]
    NoActionsRegistered(ActionEvent),

    /// Errors raised by the event processor.
    #[error("unknown error from the event processor")]
    ProcessorError(#[from] ProcessorError),

    /// Errors raised during `libinput` initialization.
    #[error("unknown error during libinput initialization")]
    LibinputError(#[from] LibinputError),
}
