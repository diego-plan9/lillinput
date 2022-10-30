//! Building blocks for mapping [`ActionEvent`]s  to [`Action`]s.

pub mod defaultcontroller;
pub mod errors;

use crate::controllers::errors::ControllerError;
use crate::events::ActionEvent;

/// Controller that connects events and actions.
pub trait Controller {
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
    fn receive_end_event(&mut self, action_event: ActionEvent) -> Result<(), ControllerError>;

    /// Run the main loop for parsing the `libinput` events.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the main loop encountered an error while polling or
    /// dispatching events.
    fn main_loop(&mut self) -> Result<(), ControllerError>;
}
