//! Components for capturing and handling events.

pub mod defaultprocessor;
pub mod errors;
pub mod libinput;

pub use crate::events::defaultprocessor::DefaultProcessor;
pub use crate::events::errors::{LibinputError, ProcessorError};

use input::event::GestureEvent;
use strum::{Display, EnumString, EnumVariantNames};
use strum_macros::EnumIter;

/// High-level application events that can trigger an action.
#[derive(
    Copy, Clone, Display, EnumIter, EnumString, EnumVariantNames, Eq, Hash, PartialEq, Debug,
)]
#[strum(serialize_all = "kebab_case")]
pub enum ActionEvent {
    /// Three-finger swipe to left.
    ThreeFingerSwipeLeft,
    /// Three-finger swipe to left-up.
    ThreeFingerSwipeLeftUp,
    /// Three-finger swipe to up.
    ThreeFingerSwipeUp,
    /// Three-finger swipe to right-up.
    ThreeFingerSwipeRightUp,
    /// Three-finger swipe to right.
    ThreeFingerSwipeRight,
    /// Three-finger swipe to right-down.
    ThreeFingerSwipeRightDown,
    /// Three-finger swipe to down.
    ThreeFingerSwipeDown,
    /// Three-finger swipe to left-down.
    ThreeFingerSwipeLeftDown,
    /// Four-finger swipe to left.
    FourFingerSwipeLeft,
    /// Four-finger swipe to left-up.
    FourFingerSwipeLeftUp,
    /// Four-finger swipe to up.
    FourFingerSwipeUp,
    /// Four-finger swipe to right-up.
    FourFingerSwipeRightUp,
    /// Four-finger swipe to right.
    FourFingerSwipeRight,
    /// Four-finger swipe to right-down.
    FourFingerSwipeRightDown,
    /// Four-finger swipe to down.
    FourFingerSwipeDown,
    /// Four-finger swipe to left-down.
    FourFingerSwipeLeftDown,
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
    /// Dispatch `libinput` events, converting them to [`ActionEvent`]s.
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

    /// Finalize a swipe gesture end event into an [`ActionEvent`].
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
    fn _end_event_to_action_event(
        &mut self,
        dx: f64,
        dy: f64,
        finger_count: i32,
    ) -> Result<ActionEvent, ProcessorError>;
}
