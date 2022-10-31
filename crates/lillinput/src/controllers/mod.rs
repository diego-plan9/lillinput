//! Building blocks for mapping [`ActionEvent`]s  to [`Action`]s.

pub mod defaultcontroller;
pub mod errors;

pub use crate::controllers::defaultcontroller::DefaultController;
pub use crate::controllers::errors::ControllerError;

use crate::events::ActionEvent;

/// Controller that connects events and actions.
pub trait Controller {
    /// Process an [`ActionEvent`], invoking the corresponding [`Action`]s.
    ///
    /// # Arguments
    ///
    /// * `dx` - the current position in the `x` axis.
    /// * `dy` - the current position in the `y` axis.
    /// * `finger_count` - the number of fingers used for the gesture.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the processing of the end of swipe event resulted in
    /// failure or in no [`Action`]s invoked.
    fn process_action_event(&mut self, action_event: ActionEvent) -> Result<(), ControllerError>;

    /// Run the main loop for parsing `libinput` events.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the main loop encountered an error while polling or
    /// dispatching events.
    fn run(&mut self) -> Result<(), ControllerError>;
}
