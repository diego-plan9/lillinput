//! Building blocks for mapping [`ActionEvent`]s  to [`Action`]s.

pub mod defaultcontroller;
pub mod errors;

use crate::controllers::errors::ControllerError;
use crate::events::ActionEvent;
use input::event::GestureEvent;

/// Controller that connects events and actions.
pub trait Controller {
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
    ) -> Result<(), ControllerError>;

    /// Receive the end of swipe gesture event.
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
    /// Returns `Err` if the processing of the end of swipe event resulted in
    /// failure or in no [`Action`]s invoked.
    fn receive_end_event(
        &mut self,
        dx: f64,
        dy: f64,
        finger_count: i32,
    ) -> Result<(), ControllerError>;

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
    ) -> Result<ActionEvent, ControllerError>;
}
