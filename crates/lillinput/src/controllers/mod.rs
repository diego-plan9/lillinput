//! Components for mapping [`ActionEvent`]s  to [`Action`]s.
//!
//! [`Action`]: crate::actions::Action

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
    /// * `action_event` - the [`ActionEvent`] to handle.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the processing of the end of swipe event resulted in
    /// failure or in no [`Action`]s invoked.
    ///
    /// [`Action`]: crate::actions::Action
    fn process_action_event(&mut self, action_event: ActionEvent) -> Result<(), ControllerError>;

    /// Run the main loop for parsing `libinput` events.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the main loop encountered an error while polling or
    /// dispatching events.
    fn run(&mut self) -> Result<(), ControllerError>;
}
