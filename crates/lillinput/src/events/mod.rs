//! Components for capturing and handling events.

pub mod defaultprocessor;
pub mod errors;
pub mod libinput;

use crate::events::errors::{LibinputError, ProcessorError};
use input::event::GestureEvent;
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
    ) -> Result<Option<ActionEvent>, ProcessorError>;

    /// Parse a swipe gesture end event into an action event.
    ///
    /// # Arguments
    ///
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

    /// Dispatch the pending `libinput` events, converting them to `ActionEvents`.
    ///
    /// # Arguments
    ///
    /// * `dx` - the current position in the `x` axis.
    /// * `dy` - the current position in the `y` axis.
    ///
    /// # Errors
    ///
    /// Returns `Err` if an error was encountered while polling of dispatching
    /// events.
    fn dispatch(&mut self, dx: &mut f64, dy: &mut f64) -> Result<Vec<ActionEvent>, LibinputError>;
}
