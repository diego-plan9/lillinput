//! Components for capturing and handling events.

pub mod errors;
pub mod libinput;

use std::os::unix::io::{AsRawFd, RawFd};

use crate::controllers::errors::ControllerError;
use crate::controllers::Controller;
use crate::events::errors::{MainLoopError, ProcessEventError};
use filedescriptor::{poll, pollfd, POLLIN};
use input::event::gesture::{
    GestureEvent, GestureEventCoordinates, GestureEventTrait, GestureSwipeEvent,
};
use input::event::Event;
use input::Libinput;
use log::debug;
use strum::{Display, EnumString, EnumVariantNames};
use strum_macros::EnumIter;

/// High-level events that can trigger an action.
#[derive(
    Copy, Clone, Display, EnumIter, EnumString, EnumVariantNames, Eq, Hash, PartialEq, Debug,
)]
#[strum(serialize_all = "kebab_case")]
pub enum ActionEvent {
    /// Three-finger swipe to left.
    ThreeFingerSwipeLeft,
    /// Three-finger swipe to right.
    ThreeFingerSwipeRight,
    /// Three-finger swipe to up.
    ThreeFingerSwipeUp,
    /// Three-finger swipe to down.
    ThreeFingerSwipeDown,
    /// Four-finger swipe to left.
    FourFingerSwipeLeft,
    /// Four-finger swipe to right.
    FourFingerSwipeRight,
    /// Four-finger swipe to up.
    FourFingerSwipeUp,
    /// Four-finger swipe to down.
    FourFingerSwipeDown,
}

/// Possible choices for finger count.
pub enum FingerCount {
    /// Three fingers.
    ThreeFinger = 3,
    /// Four fingers.
    FourFinger = 4,
}

impl TryFrom<i32> for FingerCount {
    type Error = ControllerError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(FingerCount::ThreeFinger),
            4 => Ok(FingerCount::FourFinger),
            _ => Err(ControllerError::UnsupportedFingerCount(value)),
        }
    }
}

/// Axis of a swipe action.
pub enum Axis {
    /// Horizontal (`X`) axis.
    X,
    /// Vertical (`Y`) axis.
    Y,
}

/// Process a single [`GestureEvent`].
///
/// # Arguments
///
/// * `event` - a gesture event.
/// * `dx` - the current position in the `x` axis.
/// * `dy` - the current position in the `y` axis.
/// * `controller` - the controller that will process the event.
fn process_event(
    event: GestureEvent,
    dx: &mut f64,
    dy: &mut f64,
    controller: &mut dyn Controller,
) -> Result<(), ProcessEventError> {
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
                controller.receive_end_event(*dx, *dy, event.finger_count())?;
            }
            // GestureEvent::Swipe is non-exhaustive.
            other => return Err(ProcessEventError::UnsupportedSwipeEvent(other)),
        }
    }

    Ok(())
}

/// Run the main loop for parsing the `libinput` events.
///
/// # Arguments
///
/// * `input` - the `libinput` object.
/// * `controller` - the controller that will process the event.
///
/// # Errors
///
/// Returns `Err` if the main loop encountered an error while polling or
/// dispatching events.
pub fn main_loop(
    mut input: Libinput,
    controller: &mut dyn Controller,
) -> Result<(), MainLoopError> {
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
                process_event(gesture_event, &mut dx, &mut dy, controller).unwrap_or_else(|e| {
                    debug!("Discarding event: {}", e);
                });
            }
        }
    }
}
