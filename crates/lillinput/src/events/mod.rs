//! Components for capturing and handling events.

pub mod defaultprocessor;
pub mod errors;
pub mod libinput;

use std::os::unix::io::{AsRawFd, RawFd};

use crate::controllers::Controller;
use crate::events::errors::{MainLoopError, ProcessorError};
use filedescriptor::{poll, pollfd, POLLIN};
use input::event::{Event, GestureEvent};
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
    type Error = ProcessorError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(FingerCount::ThreeFinger),
            4 => Ok(FingerCount::FourFinger),
            _ => Err(ProcessorError::UnsupportedFingerCount(value)),
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

/// Events processor, converting `libinput` events into [`ActionEvent`]s.
pub trait Processor {
    /// Process a single `libinput` [`GestureEvent`].
    ///
    /// # Arguments
    ///
    /// * `self` - controller.
    /// * `event` - a gesture event.
    /// * `dx` - the current position in the `x` axis.
    /// * `dy` - the current position in the `y` axis.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the processing of the event failed.
    fn process_event(
        &mut self,
        event: GestureEvent,
        dx: &mut f64,
        dy: &mut f64,
    ) -> Result<(), ProcessorError>;

    /// Parse a swipe gesture end event into an action event.
    ///
    /// # Arguments
    ///
    /// * `self` - controller.
    /// * `dx` - the current position in the `x` axis.
    /// * `dy` - the current position in the `y` axis.
    /// * `finger_count` - the number of fingers used for the gesture.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the processing of the swipe event did not result in a
    /// [`ActionEvent`].
    fn end_event_to_action_event(
        &mut self,
        dx: f64,
        dy: f64,
        finger_count: i32,
    ) -> Result<ActionEvent, ProcessorError>;
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
                controller
                    .process_event(gesture_event, &mut dx, &mut dy)
                    .unwrap_or_else(|e| {
                        debug!("Discarding event: {}", e);
                    });
            }
        }
    }
}
