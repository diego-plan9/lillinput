//! Errors related to [`controllers`].
//!
//! [`controllers`]: crate::controllers

use crate::events::errors::{LibinputError, ProcessorError};
use crate::events::ActionEvent;
use thiserror::Error;

/// Errors raised during processing of events in the [`Controller`].
///
/// [`Controller`]: crate::controllers::Controller
#[derive(Error, Debug)]
pub enum ControllerError {
    /// No actions registered for event.
    #[error("no actions registered for event {0}")]
    NoActionsRegistered(ActionEvent),

    /// Error raised by the event processor.
    #[error("unknown error from the event processor")]
    ProcessorError(#[from] ProcessorError),

    /// Error raised during `libinput` initialization.
    #[error("unknown error during libinput initialization")]
    LibinputError(#[from] LibinputError),
}
