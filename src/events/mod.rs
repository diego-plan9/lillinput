//! Components for capturing and handling events.

use std::io::Error as IoError;
use std::os::unix::io::{AsRawFd, RawFd};

use filedescriptor::{poll, pollfd, Error as FileDescriptorError, POLLIN};
use input::event::gesture::{
    GestureEvent, GestureEventCoordinates, GestureEventTrait, GestureSwipeEvent,
};
use input::event::Event;
use input::Libinput;

use super::actions::{ActionController, ActionMap};

pub mod libinput;

/// Process a single `GestureEvent`.
///
/// # Arguments
///
/// * `event` - a gesture event
/// * `dx` - the current position in the `x` axis
/// * `dy` - the current position in the `y` axis
/// * `action_map` - the action map that will process the event
fn process_event(event: GestureEvent, dx: &mut f64, dy: &mut f64, action_map: &mut ActionMap) {
    if let GestureEvent::Swipe(event) = event {
        match event {
            GestureSwipeEvent::Begin(_begin_event) => {
                (*dx) = 0.0;
                (*dy) = 0.0;
            }
            GestureSwipeEvent::Update(update_event) => {
                (*dx) += update_event.dx();
                (*dy) += update_event.dy();
            }
            GestureSwipeEvent::End(ref _end_event) => {
                action_map.receive_end_event(dx, dy, event.finger_count());
            }
            // GestureEvent::Swipe is non-exhaustive.
            _ => (),
        }
    }
}

/// Custom error issued during the main loop.
///
/// This custom error message captures the errors emitted during the main loop:
/// * filedescriptor::Error
/// * std::io::Error
#[derive(Debug, Clone)]
pub struct MainLoopError {
    /// Content of the original error.
    pub message: String,
}

impl From<FileDescriptorError> for MainLoopError {
    fn from(e: FileDescriptorError) -> Self {
        MainLoopError {
            message: e.to_string(),
        }
    }
}

impl From<IoError> for MainLoopError {
    fn from(e: IoError) -> Self {
        MainLoopError {
            message: e.to_string(),
        }
    }
}

/// Run the main loop for parsing the libinput events.
///
/// # Arguments
///
/// * `input` - the libinput object
/// * `action_map` - the action map that will process the event
pub fn main_loop(mut input: Libinput, action_map: &mut ActionMap) -> Result<(), MainLoopError> {
    // Variables for tracking the cursor position changes.
    let mut dx: f64 = 0.0;
    let mut dy: f64 = 0.0;

    // Use a raw file descriptor for polling.
    let raw_fd: RawFd = input.as_raw_fd();

    let mut poll_array = [pollfd {
        fd: raw_fd,
        events: POLLIN,
        revents: 0,
    }];

    loop {
        // Block until the descriptor is ready.
        poll(&mut poll_array, None)?;

        // Dispatch, bubbling up in case of an error.
        input.dispatch()?;
        for event in &mut input {
            if let Event::Gesture(gesture_event) = event {
                process_event(gesture_event, &mut dx, &mut dy, action_map);
            }
        }
    }
}
