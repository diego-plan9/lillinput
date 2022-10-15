//! Errors related to events.

use std::io::Error as IoError;

use filedescriptor::Error as FileDescriptorError;
use thiserror::Error;

/// Custom error issued during the main loop.
///
/// This custom error message captures the errors emitted during the main loop,
/// which wrap over:
/// * [`filedescriptor::Error`] (during [`filedescriptor::poll`]).
/// * [`std::io::Error`] (during [`input::Libinput::dispatch`]).
#[derive(Error, Debug)]
pub enum MainLoopError {
    #[error("unknown error while dispatching libinput event")]
    DispatchError(#[from] IoError),

    #[error("unknown error while polling the file descriptor")]
    IOError(#[from] FileDescriptorError),
}

/// Errors raised during `libinput` initialization.
#[derive(Error, Debug)]
pub enum LibinputError {
    #[error("error while assigning seat to the libinput context")]
    SeatError,
}
