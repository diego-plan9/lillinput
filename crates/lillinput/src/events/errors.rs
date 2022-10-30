//! Errors related to events.

use std::io::Error as IoError;

use filedescriptor::Error as FileDescriptorError;
use input::event::gesture::GestureSwipeEvent;
use thiserror::Error;

/// Errors raised during `libinput` initialization.
#[derive(Error, Debug)]
pub enum LibinputError {
    /// Error while assigning seat to the libinput context.
    #[error("error while assigning seat to the libinput context")]
    SeatError,

    /// Unknown error while dispatching libinput event.
    #[error("unknown error while dispatching libinput event")]
    DispatchError(#[from] IoError),

    /// Unknown error while polling for the file descriptor.
    #[error("unknown error while polling the file descriptor")]
    IOError(#[from] FileDescriptorError),
}

/// Custom error issued during the main loop.
///
/// This custom error message captures the errors emitted during the main loop,
/// which wrap over:
/// * [`filedescriptor::Error`] (during [`filedescriptor::poll`]).
/// * [`std::io::Error`] (during [`input::Libinput::dispatch`]).
#[derive(Error, Debug)]
pub enum MainLoopError {
    /// Unknown error while dispatching libinput event.
    #[error("unknown error while dispatching libinput event")]
    DispatchError(#[from] IoError),

    /// Unknown error while polling for the file descriptor.
    #[error("unknown error while polling the file descriptor")]
    IOError(#[from] FileDescriptorError),
}

/// Errors raised during processing of events in the processor.
#[derive(Error, Debug)]
pub enum ProcessorError {
    /// Unsupported finger count.
    #[error("unsupported finger count ({0})")]
    UnsupportedFingerCount(i32),

    /// Unsupported swipe event.
    #[error("unsupported swipe event ({:?})", .0)]
    UnsupportedSwipeEvent(GestureSwipeEvent),

    /// Event displacement is below threshold.
    #[error("event displacement is below threshold ({0})")]
    DisplacementBelowThreshold(f64),

    /// Error while assigning seat to the libinput context.
    #[error("error while assigning seat to the libinput context")]
    SeatError,
}
